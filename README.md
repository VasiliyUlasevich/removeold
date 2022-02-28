# RmoveOld

This tool helps to get some extra free space by deleting old files.

## Description

BE CAREFUL! This tool is extremely dangerous. Because it seeks files from a current start point (The directory where you start this tool).
The tool in the process of searching makes a list of files(which contains name, size and date of creation of the file), sorts this list by the file create date and tries to delete the oldest files until the amount of free space is greater than the "size" parameter (You should specify this option on the command line).

## Command line parameters

```  help - show this help
  size=number - if number is greater then free space, the program will search for and deleting the oldest files. This parameter is required.
  ext=extension - if specify this parameter, the application will only look for files with that extension.
  log=log_file_name - if specify this parameter, the application will save the names of deleted files to this log file.
  dry - process without real files deleting
```