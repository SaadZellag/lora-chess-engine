cd training

python trainer.py \
    --train-input ../training-data/training.binpack \
    --test-input ../training-data/testing.binpack \
    --num-workers 1 \
    --dll-path ../lora/target/release/libdataloader.so \
    --checkpoint tb_logs/NNUE-2x32-32/2026-08-24-10-26-45/checkpoints/NNUE-2x32-32-2026-08-24-10-26-45.ckpt \
    --epochs 1000 \
    --batch-size 65536 \
    --matmul-precision medium

