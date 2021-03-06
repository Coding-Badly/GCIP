/*==============================================================================

  GCIP - G Code Insert Pause
  
  This program inserts a pause in G-code generated by KISSlicer for a
  PolyPrinter.  Inputs to the program are the path to the G-code file and the
  part height in millimeters where the pause is to be inserted.  The file is
  modified in place using a two-phase commit.

  ----------------------------------------------------------------------------

  Copyright 2019 Brian Cook (aka Coding-Badly)

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.

==============================================================================*/
use std::fs::File;
use std::fs::remove_file;
use std::fs::rename;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::path::Path;

use regex::Regex;
use structopt::StructOpt;


fn build_pause_gcode(maximum_extruder_temperature: f64, extruder_temperature: f64, resume_height: f64) -> String {
    return format!("\r
;===============================================================================\r
;-------------------------------------------------------------------------------\r
\r
; PAUSE for filament change.\r
\r
; The human may be a great while so set the extruder temperature to 0 C.\r
; Set Extruder Temperature\r
; http://reprap.org/wiki/G-code#M104:_Set_Extruder_Temperature\r
M104 S0\r
\r
; Set to Relative Positioning\r
; http://reprap.org/wiki/G-code#G91:_Set_to_Relative_Positioning\r
G91\r
\r
; Retract the filament a bit so it does not dribble.  (Extrude -2 mm at a\r
; feedrate of 500.)\r
; Linear Move\r
; http://reprap.org/wiki/G-code#G0_.26_G1:_Move\r
G1 E-2.000000 F500\r
\r
; Move the gantry up 1 mm to clear the part\r
; Rapid Move\r
; http://reprap.org/wiki/G-code#G0_.26_G1:_Move\r
G0 Z1\r
\r
; Set to Absolute Positioning\r
; http://reprap.org/wiki/G-code#G90:_Set_to_Absolute_Positioning\r
G90\r
\r
; Move to a safe location.\r
; Rapid Move\r
; http://reprap.org/wiki/G-code#G0_.26_G1:_Move\r
G0 X0 Y0\r
G0 Z30\r
\r
; Ask the human to take action.\r
; Display Message\r
; https://reprap.org/wiki/G-code#M117:_Display_Message\r
M117 Change filament then resume.\r
\r
; Back out the filament all the way.\r
G91\r
G1 E-50 F500\r
G90\r
G92 E0\r
\r
; Ask OctoPrint for a pause.  The human has to resume.\r
; Stop or Unconditional stop\r
; https://reprap.org/wiki/G-code#M0:_Stop_or_Unconditional_stop\r
M0\r
\r
; Heat the extruder so the previous color filament can be flushed.\r
; Set Extruder Temperature and Wait\r
; http://reprap.org/wiki/G-code#M109:_Set_Extruder_Temperature_and_Wait\r
M109 S260\r
\r
; Set to Relative Positioning\r
; http://reprap.org/wiki/G-code#G91:_Set_to_Relative_Positioning\r
G91\r
\r
; Extrude 75 mm at a feedrate of 500 to clear the previous color.\r
; Linear Move\r
; http://reprap.org/wiki/G-code#G0_.26_G1:_Move\r
G1 E75 F500\r
\r
; Ask the human to take action.\r
; Display Message\r
; https://reprap.org/wiki/G-code#M117:_Display_Message\r
M117 Clean the mess, adjust, then resume.\r
\r
; The human may be a while so set the extruder temperature to 180 C.\r
; Set Extruder Temperature and Wait\r
; http://reprap.org/wiki/G-code#M109:_Set_Extruder_Temperature_and_Wait\r
M109 S180\r
\r
; Ask OctoPrint for another pause so the human can clear the waste / make any\r
; further manual adjustments.\r
; Stop or Unconditional stop\r
; https://reprap.org/wiki/G-code#M0:_Stop_or_Unconditional_stop\r
M0\r
\r
; Set to Absolute Positioning.\r
; http://reprap.org/wiki/G-code#G90:_Set_to_Absolute_Positioning\r
G90\r
\r
; Reset extruder position.\r
; Set Position\r
; https://reprap.org/wiki/G-code#G92:_Set_Position\r
G92 E0\r
\r
; Standard PolyPrinter wipe.  (Mostly) copied from above.\r
\r
; absolute mode\r
G90\r
; G92 Z0\r
; G1 Z3 F240 ; raise head a bit in case was stopped in the middle of something and made a bump\r
G28 Y0  ; home Y axis  to put bed as far back as possible, over the power supply, and so the cooling blower doesn't cool the bed\r
; M104 S180; start it warming but don't make it drool\r
; M190  S110; wait for bed to warm up\r
G28 X0	; signal that we are just waiting for the head now\r
; try to get the head as close as possible to the final first-layer temp as possible, to avoid making a puddle at the home position\r
M109 S{}; ensure melting before Z homing\r
G28 Z0 ; home Z\r
; sitting at home position\r
; just in case it melted through and touched, raise head a little before moving, to avoid triggering a bed touch that kills the print\r
G1 Z.50 F240; keep head from melting through tape and causing spurious head contact with bed\r
G1 X10 Y5 Z.25 F2000\r
G92 E0\r
G1 X10 Y0 F1000\r
G1 E12 F300\r
G1 Z3  F240\r
G1 X20 Y5 Z.10 F1000\r
G1 X20 Y0 F2000\r
G1 Z.25  F240\r
G1 E15 F300\r
G1 Z3  F240\r
G1 X30 Y5 Z.10 F1000\r
; retract just a little for the move to the part\r
G92 E0\r
G1 E-.5 F300\r
G1 X30 Y0  F2000\r
M104 S{}; restore the extruder temperature\r
; G1 Z.25 F240 ; raise to first layer height to avoid tracking across bed\r
\r
G0 Z{:.3} ; <--- Change the value after 'Z' to a bit more than the next layer height. ---<\r
\r
; Reset extruder position.\r
; Set Position\r
; https://reprap.org/wiki/G-code#G92:_Set_Position\r
G92 E0\r
\r
;-------------------------------------------------------------------------------\r
;===============================================================================\r
\r
", maximum_extruder_temperature, extruder_temperature, resume_height);
}


trait FixedPointComparable {
    fn comparable(&self) -> i64;
}

impl FixedPointComparable for f64 {
    fn comparable(&self) -> i64 {
        return (((self * 10000.0) as i64) + 5) / 10;
    }
}

impl FixedPointComparable for str {
    fn comparable(&self) -> i64 {
        return (self.parse::<f64>().unwrap()).comparable()
    }
}


fn get_pause_height_from_stdin() -> Result<String, std::io::Error> {
    print!("Pause at what height? ");
    stdout().flush()?;
    let mut raw = String::new();
    stdin().read_line(&mut raw)?;
    Ok(raw)
}

static PAUSE_HEIGHT_NOT_NUMBER: &str = "Pause Height must be a positive number greater than zero.";

fn get_pause_height(arguments: &GcipArguments) -> Result<f64,  Box<dyn std::error::Error>> {

    let pause_height_string = match arguments.pause_height {
        Some(x) => x.to_string(),
        None    => get_pause_height_from_stdin()?
    };
    let re_ph = Regex::new(r"([-]?[0-9]+[.]?[0-9]*)")?;

    let captures = re_ph.captures(&pause_height_string);

    if let Some(ref captures) = captures {

        let pause_height = captures[1].parse::<f64>()?;
        if pause_height <= 0.0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, PAUSE_HEIGHT_NOT_NUMBER)));
        }
        else {
            return Ok(pause_height);
        }
    }
    else {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, PAUSE_HEIGHT_NOT_NUMBER)));
    }
}


#[derive(StructOpt)]
#[structopt(about="This program inserts a pause in G-code generated by KISSlicer for a \
PolyPrinter.  Inputs to the program are the path to the G-code file and the part height in \
millimeters where the pause is to be inserted.  The file is modified in place using a two-phase \
commit.")]
struct GcipArguments {
    //use_standard_input: bool,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    #[structopt(short="h", long="height", help="Print height (millimeters) where a pause is inserted.")]
    pause_height: Option<f64>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let arguments = GcipArguments::from_args();

    let path_src = Path::new(&arguments.path);

    let pause_height = get_pause_height(&arguments)?;
    let pause_height_comparable = pause_height.comparable();

    // Scan for...
    // ...END_LAYER_OBJECT comments.
    let re_elo = Regex::new(r"^;\s+END_LAYER_OBJECT\s+z\s*=([0-9]+[.]?[0-9]*)(.*)$")?;
    // ...set extruder temperature G code
    let re_set = Regex::new(r"^ *M10[49] *S([0-9]+)")?;

    // Rust prefers LF line endings.  We're going to use CR LF.
    let crlf = "\r\n".as_bytes();

    // Build a Path for the temporary file and the backup file
    let t1 = path_src.to_str().unwrap();
    let t2 = format!("{}.tmp", t1);
    let path_tmp = Path::new(&t2);
    let t3 = format!("{}.bak", t1);
    let path_bak = Path::new(&t3);

    // Try to open the input file.
    let inf = File::open(path_src)?;

    // Added to the END_LAYER_OBJECT line to mark a pause.
    const PAUSE_HERE: &str = "  (pause here)";

    let mut commit = false;
    {
        // Try to open the output file.
        let mut ouf = File::create(path_tmp)?;

        // Read the input file line-by-line
        let inf = BufReader::new(&inf);

        let mut target_layer_found = false;
        
        let mut current_extruder_temperature = 0.0;
        let mut maximum_extruder_temperature = 0.0;
        
        // For each line in the input file.
        for wrapped_line in inf.lines() {
        
            let line = wrapped_line?;
            let line_as_bytes = line.as_bytes();
            let mut write_original = true;

            if line.contains("END_LAYER_OBJECT") {
                let captures = re_elo.captures(&line);
                if let Some(ref captures) = captures {
                    let z = captures[1].parse::<f64>()?;
                    let z_comparable = z.comparable();
                    if z_comparable == pause_height_comparable {
                        if &captures[2] != PAUSE_HERE {
                            write_original = false;
                            ouf.write(line_as_bytes)?;
                            ouf.write(PAUSE_HERE.as_bytes())?;
                            ouf.write(crlf)?;
                            let t1 = build_pause_gcode(maximum_extruder_temperature, current_extruder_temperature, pause_height + 0.500);
                            let t2 = t1.as_bytes();
                            ouf.write(t2)?;
                            eprintln!("INFO: Pause inserted after the layer at {:0.3} is finished.", pause_height);
                            commit = true;
                        }
                        else {
                            eprintln!("WARNING: There is already a pause at {:0.3}.", pause_height);
                        }
                        target_layer_found = true;
                    }
                    else if z_comparable > pause_height_comparable {
                        if ! target_layer_found {
                            eprintln!("WARNING: There is not a layer that ends at {:0.3}.", pause_height);
                            target_layer_found = true;
                        }
                    }
                }
            }
            else if line.starts_with("M10") {
                let captures = re_set.captures(&line);
                if let Some(ref captures) = captures {
                    current_extruder_temperature = captures[1].parse::<f64>()?;
                    if current_extruder_temperature > maximum_extruder_temperature {
                        maximum_extruder_temperature = current_extruder_temperature;
                    }
                    // rmv println!("{}", line);
                    // rmv println!("current_extruder_temperature = {}", current_extruder_temperature);
                    // rmv println!("maximum_extruder_temperature = {}", maximum_extruder_temperature);
                }
            }

            if write_original {
                ouf.write(line_as_bytes)?;
                ouf.write(crlf)?;
            }
        }
    }
    if commit {
        let _ = remove_file(path_bak);
        rename(path_src, path_bak)?;
        rename(path_tmp, path_src)?;
    }
    else {
        let _ = remove_file(path_tmp);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_arguments_001() {
        // let raw = vec!["program_name_goes_here.exe", "junk.gcode"];
        let raw: std::vec::Vec::<&str> = vec![""];
        let arguments = GcipArguments::from_iter_safe(raw.iter());
        assert!(arguments.is_err());
    }

    #[test]
    fn bad_arguments_002() {
        let raw = vec!["program_name_goes_here.exe"];
        let arguments = GcipArguments::from_iter_safe(raw.iter());
        assert!(arguments.is_err());
    }

    #[test]
    fn good_arguments_003() {
        let raw = vec!["program_name_goes_here.exe", "test.gc"];
        let arguments = GcipArguments::from_iter_safe(raw.iter());
        assert!(arguments.is_ok());
        if let Ok(arguments) = arguments {
            let right = std::path::PathBuf::from("test.gc");
            assert!(arguments.path == right);
        }
    }
}

