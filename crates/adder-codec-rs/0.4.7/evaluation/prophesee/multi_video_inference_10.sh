INPUT_DIR="/media/andrew/One Touch/HMNet/data/gen1_subset/source/detection_dataset_duration_60s_ratio_1.0/test/"
cd "$INPUT_DIR"

for FILE in *.dat; 
do 

  cd /home/andrew/Code/adder-codec-rs/adder-codec-rs/evaluation/prophesee

  input_dat="$INPUT_DIR$FILE"
  echo $input_dat; 

  rm /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/data/gen1/source/detection_dataset_duration_60s_ratio_1.0/test/*.dat
  rm /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/data/gen1/test_evt/*.npy
  rm /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/workspace/hmnet_B3_yolox_tbptt/result/pred_test/*.npy

  # Extract the filename without extension
  filename="${input_dat##*/}"   # Double ## for removing everything before the last slash

  # Generate output filenames
  adder_out="/home/andrew/Downloads/hmnet_testing2/${filename%.*}.adder"  # ${filename%.*} removes extension
  final_adder_dat_out="/home/andrew/Downloads/hmnet_adder_10/${filename%.*}.dat"
  final_shrink_dat_out="/home/andrew/Downloads/hmnet_shrunk_10/${filename%.*}.dat"
  dat_out="/media/andrew/One Touch/HMNET2_autotest2/experiments/detection/data/gen1/source/detection_dataset_duration_60s_ratio_1.0/test/${filename%.*}.dat"

  cd /home/andrew/Code/adder-codec-rs && cargo run --release --bin prophesee_to_adder -- --ref-time 20 --delta-t-max 40 --input "$input_dat" --output "$dat_out" --crf 3 --features --feature-radius 10
  # cd /home/andrew/Code/adder-codec-rs/adder-to-dvs && cargo run --release -- --input "$adder_out" --output-events "$dat_out" --theta 0.004

  # Run HMNET on the GT .dat

  export PYTHONPATH=$PYTHONPATH:"/media/andrew/One Touch/HMNET2_crf0"

  cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/data/gen1 && echo $(pwd) && bash ./scripts/prepair.sh
  cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection && python ./scripts/test.py ./config/hmnet_B3_yolox_tbptt.py ./data/gen1/list/test/ ./data/gen1/ --pretrained ./pretrained/gen1_hmnet_B3_tbptt.pth --speed_test
  cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection
  data=$(./scripts/run_eval.sh ./config/hmnet_B3_yolox_tbptt.py)  # This line is optional if you don't need the entire output
  echo "$data"

  mapline=$(echo "$data" | grep 'Average Precision' | head -1)
  mapadder=$(echo "$mapline" | rev | cut -d ' ' -f 1 | rev)
  if [[ "$mapadder" =~ ^[-+]?[0-9]+(\.[0-9]+)?$ ]]; then
    # Check if it's a valid number format
    echo "$mapadder"
  else
    echo "Error: Last word is not a valid number."
  fi

  cp "$dat_out" "$final_adder_dat_out"

  for i in {1..2}
  do
      rm /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/data/gen1/test_evt/*.npy
      rm /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/workspace/hmnet_B3_yolox_tbptt/result/pred_test/*.npy

      cd /home/andrew/Code/adder-codec-rs/adder-codec-rs/evaluation/prophesee && python single_video_shrink.py "$input_dat" "$dat_out" "$dat_out"
      cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection/data/gen1 && echo $(pwd) && bash ./scripts/prepair.sh
      cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection && python ./scripts/test.py ./config/hmnet_B3_yolox_tbptt.py ./data/gen1/list/test/ ./data/gen1/ --pretrained ./pretrained/gen1_hmnet_B3_tbptt.pth --speed_test
      cd /media/andrew/One\ Touch/HMNET2_autotest2/experiments/detection
      data=$(./scripts/run_eval.sh ./config/hmnet_B3_yolox_tbptt.py)  # This line is optional if you don't need the entire output
      echo "$data"

      mapline=$(echo "$data" | grep 'Average Precision' | head -1)
      mapshrink=$(echo "$mapline" | rev | cut -d ' ' -f 1 | rev)
      if [[ "$mapshrink" =~ ^[-+]?[0-9]+(\.[0-9]+)?$ ]]; then
      # Check if it's a valid number format
      echo "$mapadder"
      echo "$mapshrink"
      else
      echo "Error: Last word is not a valid number."
      fi

      if [[ $mapshrink < $mapadder ]]; then
      echo "number1 (as string) is less than number2 (as string)"
      break
      else
      echo "number1 (as string) is not less than number2 (as string)"
      fi
  done

  cp "$dat_out" "$final_shrink_dat_out"
done
