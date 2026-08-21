How to use jmp:
On its own, rust cannot change your current directory, so you will need to get some bash involved.
1) open your .bashrc, commonly in your home folder ~/.bashrc
2) add this block:
	jmp() {
    		cd "$(path/to/AJ_jmp/target/release/AJ_jmp "$1" "$2")"
	}
Change "path/to/" to your filepath running up to the jmp project folder
3) save your .bashrc
	source .bashrc

Now when you want to run it, simply type from anywhere jmp <Pattern> <Path>
The path can be relative or full.

Example:
jmp goal.txt /C/Some/File/Path
this will take you to the directory containing the file goal.txt

Limitation: This does not work "up" the filetree, if you are trying to jump to a folder outside of the folder you are running it from, you will need to give some sort of starting file path, you cant just start from ../../ or so on.
