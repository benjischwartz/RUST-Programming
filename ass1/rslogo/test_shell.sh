#!/bin/sh

count=0
num_files=40

for file in logo_examples/*; do
    if [ -f "$file" ]; then
        echo "Processing file: $file"
        6991 cargo run -- $file output1.svg 200 200 >/dev/null 2>error_log.txt
        6991 rslogo $file output2.svg 200 200 >/dev/null 2>>error_log.txt

		if [ ! -f "output2.svg" ] && [ -f "output1.svg" ]; then
			echo "output1.svg exists but output2.svg doesn't exist."

		elif [ -f "output2.svg" ] && [ ! -f "output1.svg" ]; then
			echo "output2.svg exists but output1.svg doesn't exist."

		elif [ -f "output1.svg" ] && [ -f "output2.svg" ]; then
			diff_output=$(diff output1.svg output2.svg)
			if [ -n "$diff_output" ]; then
			
				echo "Diff: "
				echo "$diff_output"
			fi
			rm output1.svg
			rm output2.svg
		fi

		count=$((count + 1))
		if [ $count -eq $num_files ]; then
			break
		fi
    fi
done

