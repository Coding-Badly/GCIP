# GCIP
G Code Insert Pause

This program inserts a pause in G-code generated by KISSlicer for a
PolyPrinter.  Inputs to the program are the path to the G-code file and the
part height in millimeters where the pause is to be inserted.  The file is
modified in place using a two-phase commit.
