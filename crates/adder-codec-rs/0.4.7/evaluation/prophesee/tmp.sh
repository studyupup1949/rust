INPUT_DIR="/media/andrew/One Touch/HMNet/data/gen1_subset/source/detection_dataset_duration_60s_ratio_1.0/test/"
cd "$INPUT_DIR"

for FILE in *.dat; 
do 

  cd /home/andrew/Code/adder-codec-rs/adder-codec-rs/evaluation/prophesee

  input_dat="$INPUT_DIR$FILE"
  echo $input_dat; 
done