#!/bin/sh

# EDIT THEESE PARAMETERS TO FIT YOUR NEED
pis_ip="192.168.43.62"
pis_pass=5ace45ad9d #THIS IS FOR THE PASSWORD FOR THE PI
pis_user=pi #THIS IS THE USERNAME ON THE PI
pis_path=/home/$pis_user #THIS IS THE DIRECTORY THAT THE PROGRAM WILL BE DROPPED IN
program_name=rust_template #THIS IS THE NAME OF THE RUST PROGRAM FOUND IN DEBUG


# scrip recuires sshpass, run sudo apt-get install sshpass
echo "attemting to build program $program_name and moving it to $pis_ip:$pis_path"
if cargo build --target="arm-unknown-linux-gnueabihf"; then
	echo "Build succesfull, sending file over"
	if sshpass -p $pis_pass scp target/arm-unknown-linux-gnueabihf/debug/$program_name $pis_user@$pis_ip:$pis_path; then
		echo "Program uploaded, running"
		echo "------------------------------------------"
		if sshpass -p $pis_pass ssh -x -t $pis_user@$pis_ip $pis_path/$program_name; then
		echo "------------------------------------------"
		echo "Program stopped"
		fi
	fi
fi
