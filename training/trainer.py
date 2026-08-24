
from io import TextIOWrapper
import subprocess
import torch
from torch import nn
import pytorch_lightning as pl
from pytorch_lightning.callbacks import ModelCheckpoint
from pytorch_lightning.loggers import TensorBoardLogger
import numpy as np
from torch.utils.data import DataLoader, Dataset
from torch.optim import Adam, SGD
from torch.optim.lr_scheduler import OneCycleLR, ReduceLROnPlateau
import torch.nn.functional as F
import struct
import sys
import argparse
from consts import *
from datetime import datetime

from nnue_dataset import SparseBatchDataset



class NNUE(pl.LightningModule):
    def __init__(self):
        super(NNUE, self).__init__()

        self.ft = nn.Linear(NUM_FEATURES, M)
        self.l1 = nn.Linear(2 * M, K)
        # self.l2 = nn.Linear(K, K)
        self.output = nn.Linear(K, 1)

    def forward(self, our_features, their_features, dim=1, debug=False):
        ours = self.ft(our_features)
        theirs = self.ft(their_features)

        accumulator = torch.cat([ours, theirs], dim=dim)

        if debug:
            print('acc:', accumulator.shape)

        l1_x = torch.clamp(accumulator, 0, 1)
        l1_out = self.l1(l1_x)

        # l2_x = torch.clamp(l1_out, 0, 1)
        # l2_out = self.l2(l2_x)

        output_x = torch.clamp(l1_out, 0, 1)

        return self.output(output_x)

    def training_step(self, batch, _):
        our_features, their_features, y = batch
        y_hat = torch.sigmoid(
            self(our_features, their_features))
        loss = F.mse_loss(y_hat, y)

        self.log('loss', loss, on_step=False, on_epoch=True)

        return loss

    def validation_step(self, batch, _):
        our_features, their_features, y = batch
        y_hat = torch.sigmoid(self(our_features, their_features))
        loss = F.mse_loss(y_hat, y)

        self.log('val_loss', loss, on_step=False, on_epoch=True, prog_bar=True)

        return loss

    def configure_optimizers(self):
        optimizer = Adam(self.parameters(), lr=LR)
        scheduler = OneCycleLR(
            optimizer,
            max_lr=1e-3,
            total_steps=int(self.trainer.estimated_stepping_batches),
            pct_start=0.05,  # 5% Warmup
            div_factor=10.0,
            final_div_factor=100.0
        )
        return {
            "optimizer": optimizer,
            "lr_scheduler": {
                "scheduler": scheduler,
                "interval": "step"
            },
            "monitor": "val_loss"
        }

    def on_before_zero_grad(self, *args, **kwargs):
        super().on_before_zero_grad(*args, **kwargs)

        to_clip = [
            # self.ft.weight,
            self.l1.weight,
            # self.l2.weight,
            self.output.weight
        ]


        with torch.no_grad():
            for clip in to_clip:
                p_data_fp32 = clip.data
                p_data_fp32.clamp_(MIN, MAX)
                clip.data.copy_(p_data_fp32)

    # def training_epoch_end(self, outputs):
    #     if self.current_epoch % 25 == 0:
    #         nnue_to_rust(self)
    #         out = subprocess.run(
    #             ["cargo", "run", "--release"],
    #             stdout=subprocess.PIPE,
    #             stderr=subprocess.DEVNULL,
    #             text=True
    #         ).stdout
    #         nodes = float(out.split()[0])
    #         print(f"bench: {out}")

    #     # self.log("nodes", nodes, on_step=False, on_epoch=True)


def load_nnue(path):
    try:
        nnue = NNUE.load_from_checkpoint(path)
    except RuntimeError:
        global M, NUM_FEATURES, K
        # Model configuration does not match
        model = torch.load(path)['state_dict']
        (M, NUM_FEATURES) = model['ft.weight'].shape
        K = model['l1.weight'].shape[0]
        nnue = NNUE.load_from_checkpoint(path)

    return nnue


if __name__ == '__main__':
    
    # Argument parser for CLI options
    parser = argparse.ArgumentParser(description='NNUE Training')
    parser.add_argument('--train-input', type=str,
                        help='Path to training data file')
    parser.add_argument('--test-input', type=str,
                        help='Path to test data file')
    parser.add_argument('--batch-size', type=int, default=8192,
                        help='Batch size for training (default: 8192)')
    parser.add_argument('--num-workers', type=int, default=1,
                        help='Number of workers for data loading (default: 1)')
    parser.add_argument('--epochs', type=int, default=50,
                        help='Number of epochs for training (default: 50)')
    parser.add_argument('--checkpoint', type=str, default=None,
                        help='Path to checkpoint for resuming training')
    parser.add_argument('--device', type=str, default='cuda')
    parser.add_argument('--matmul-precision', type=str, default=None,
                        help='Matmul precision for training (default: None)')
    parser.add_argument('--dll-path', type=str)
    args, unknown = parser.parse_known_args()


    # train_dataset = ChessDataSet('../games/training_data.bin')
    # train_dataloader = DataLoader(train_dataset, batch_size=BATCH_SIZE,
    #                               shuffle=True, num_workers=NUM_WORKERS)

    # val_dataset = ChessDataSet('../games/val_training_data.bin')
    # val_dataloader = DataLoader(
    #     val_dataset, batch_size=BATCH_SIZE, num_workers=NUM_WORKERS)

    if args.matmul_precision:
        torch.set_float32_matmul_precision(args.matmul_precision)

    now = datetime.now().strftime("%Y-%m-%d-%H-%M-%S")

    checkpoint = ModelCheckpoint(
        save_top_k=1,
        monitor='val_loss',
        filename=f'NNUE-2x{M}-{K}-{now}'
    )

    tb_logger = TensorBoardLogger("tb_logs", name=f"NNUE-2x{M}-{K}", version=now)

    # pytorch_profiler = PyTorchProfiler(
    #     dirpath="tb_logs/profiler",
    #     filename="perf_logs",
    #     record_shapes=True,
    #     profile_memory=True,
    #     trace_cuda=True
    # )

    trainer = pl.Trainer(callbacks=checkpoint,
                         logger=tb_logger,
                         log_every_n_steps=1,
                         accelerator='gpu', devices=1,
                         max_epochs=args.epochs,
                        #  profiler="advanced"
                         )

    # if args.checkpoint:
    #     nnue = load_nnue(args.checkpoint)
    # else:
    nnue = NNUE()
    nnue = nnue.to(args.device)
    # nnue = NNUE()
    # try:
    #     nnue = torch.compile(nnue)
    # except Exception as e:
    #     print(f"Warning: torch.compile failed with error: {e}. Proceeding without compilation.")

    train_dataset = SparseBatchDataset(args.dll_path, args.train_input, args.batch_size)
    val_dataset = SparseBatchDataset(args.dll_path, args.test_input, args.batch_size)
    train_dataloader = DataLoader(train_dataset, batch_size=None, num_workers=0, pin_memory=True)
    val_dataloader = DataLoader(val_dataset, batch_size=None, num_workers=0, pin_memory=True)

    trainer.fit(nnue, train_dataloaders=train_dataloader, val_dataloaders=val_dataloader, ckpt_path=args.checkpoint)

# elif sys.argv[1] == 'convert':
#     nnue = load_nnue(sys.argv[2])

#     nnue_to_rust(nnue)
#     model_params = sum([par.numel() for par in nnue.parameters()])
#     print(f"Total model parameters: {model_params}")