cd training

python trainer.py \
    --train-input ../training-data/training.binpack \
    --test-input ../training-data/testing.binpack \
    --num-workers 4 \
    --dll-path ../lora/target/release/libdataloader.so \
    --epochs 1000 \
    --batch-size 65536 \
    --matmul-precision medium \
    --step-size 4 # --checkpoint tb_logs/NNUE-2x32-32/2026-08-24-22-12-10/checkpoints/NNUE-2x32-32-2026-08-24-22-12-10.ckpt

# python trainer.py \
#     --train-input ../training-data/debug_entry.binpack \
#     --test-input ../training-data/debug_entry.binpack \
#     --num-workers 1 \
#     --dll-path ../lora/target/release/libdataloader.so \
#     --epochs 100 \
#     --batch-size 1 \
#     --matmul-precision medium