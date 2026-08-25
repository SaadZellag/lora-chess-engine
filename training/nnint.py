import argparse
import json
import math
import matplotlib.pyplot as plt
import sys
import numpy as np
from trainer import NNUE, load_nnue
from consts import *
from nnue_dataset import SparseBatchDataset
import torch


def to_np(tensor):
    return tensor.cpu().detach().to_dense().numpy()


def tensor_to_list(tensor, scale, type=np.int32):
    tensor = tensor.cpu().detach().numpy()
    tensor = np.round(tensor * scale)
    tensor = tensor.astype(type, order='C')
    return tensor.tolist()


class intNNUE():
    def __init__(self, nnue: NNUE):
        def get_tensor(tensor, scale):
            tensor = to_np(tensor)
            tensor = np.round(tensor * scale).astype(np.int32)
            return tensor

        self.ft_weight = get_tensor(nnue.ft.weight, FEATURE_SCALE)
        self.ft_bias = get_tensor(nnue.ft.bias,   FEATURE_BIAS_SCALE)
        self.l1_weight = get_tensor(nnue.l1.weight, WEIGHT_SCALE)
        self.l1_bias = get_tensor(nnue.l1.bias,   BIAS_SCALE)
        # self.l2_weight = get_tensor(nnue.l2.weight, WEIGHT_SCALE)
        # self.l2_bias = get_tensor(nnue.l2.bias,   BIAS_SCALE)
        self.output_weight = get_tensor(nnue.output.weight, WEIGHT_SCALE)
        self.output_bias = get_tensor(nnue.output.bias, BIAS_SCALE)

        self.ft_bias = self.ft_bias.reshape(-1, 1)
        self.l1_bias = self.l1_bias.reshape(-1, 1)
        self.output_bias = self.output_bias.reshape(-1, 1)

    def forward(self, our_features, their_features):

        our_features = np.transpose(to_np(our_features))
        their_features = np.transpose(to_np(their_features))

        ours = np.dot(self.ft_weight, our_features) + self.ft_bias
        theirs = np.dot(self.ft_weight, their_features) + self.ft_bias

        acc = np.concatenate([ours, theirs], axis=0)

        # print('int acc:', acc)

        l1_x = np.clip(acc // FEATURE_CRELU_DOWNSCALE, 0, 127)
        l1 = np.dot(self.l1_weight, l1_x) + self.l1_bias

        # l2_x = np.clip(l1 // WEIGHT_SCALE, 0, 127)

        # print('int l2_x:', l2_x)
        # l2 = np.dot(self.l2_weight, l2_x) + self.l2_bias

        output_x = np.clip(l1 // REGULAR_CRELU_DOWNSCALE, 0, 127)
        output = np.dot(self.output_weight, output_x) + self.output_bias

        return output / 8


def output_to_rust(nnue: NNUE, output_file):

    INACTIVE_THRESHOLD = 16

    feature_weights = tensor_to_list(nnue.ft.weight.transpose(0, 1), FEATURE_SCALE, np.int16)
    feature_bias = tensor_to_list(nnue.ft.bias, FEATURE_BIAS_SCALE, np.int16)

    number_of_feature_weights = len(feature_weights) * len(feature_weights[0])
    number_of_feature_inactive = 0
    # Count the number of 0s to know how "fully trained the NN is"
    for row in feature_weights:
        for value in row:
            if abs(value) < INACTIVE_THRESHOLD:
                number_of_feature_inactive += 1

    percent_feature_weights_zero = number_of_feature_inactive / number_of_feature_weights * 100

    print(f"Feature weights: {number_of_feature_weights} total, {number_of_feature_inactive} inactive ({percent_feature_weights_zero:.2f}%)")

    feature_weights_flat = [v for row in feature_weights for v in row]
    print(f"Mean: {np.mean(feature_weights_flat):.4f}, Median: {np.median(feature_weights_flat):.4f}, Std Dev: {np.std(feature_weights_flat):.4f}")
    print(f'Max: {np.max(feature_weights_flat)}, Min: {np.min(feature_weights_flat)}')

    with open(output_file, 'w') as f:
        f.write(f'''
            pub static EVALUATOR: NNUE = NNUE {{
                ft: FeatureLayer {{
                    weights: {feature_weights},
                    bias: {feature_bias}
                }},
                layer_1: Layer {{
                    weights: {tensor_to_list(nnue.l1.weight, WEIGHT_SCALE, np.int8)},
                    bias: {tensor_to_list(nnue.l1.bias, BIAS_SCALE, np.int32)}
                }},
                output: Layer {{
                    weights: {tensor_to_list(nnue.output.weight, WEIGHT_SCALE, np.int8)},
                    bias: {tensor_to_list(nnue.output.bias, BIAS_SCALE, np.int32)}
                }}
            }};
        ''')
        


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='NNUE Export')
    parser.add_argument('--input', type=str, required=True,
                        help='Path to the input NNUE model checkpoint')
    parser.add_argument('--output', type=str, required=True,
                        help='Path to the output JSON file')
    args = parser.parse_args()

    nnue = load_nnue(args.input)
    output_to_rust(nnue, args.output)

    # dataset = SparseBatchDataset(b'../games/val_training_data.bin', 8192)
    # nnue = load_nnue(sys.argv[1])
    # intnnue = intNNUE(nnue)

    # # print('weights', intnnue.ft_weight)
    # # print('bias', intnnue.ft_bias)
    # # print('Scale:', SCALING_FACTOR)

    # data_x = []
    # data_y = []

    # total_batches = 100000

    # try:
    #     for (i, data) in enumerate(dataset):
    #         our, their, y_batch = data

    #         # y_hat_batch = nnue.forward(our, their, dim=1, debug=False)
    #         y_int_hat_batch = intnnue.forward(our, their)[0]

    #         DELTA = 1e-20
    #         y_batch = torch.clamp(y_batch, DELTA, 1-DELTA)
    #         y_batch = torch.log(y_batch/(1-y_batch)) * \
    #             WEIGHT_SCALE * ACTIVATION_RANGE / 8

    #         data_x += y_int_hat_batch.tolist()
    #         # data_x += torch.sigmoid(y_hat_batch).reshape(-1).tolist()
    #         data_y += y_batch.reshape(-1).tolist()

    #         # for (j, (y, y_int_hat)) in enumerate(zip(y_batch, y_int_hat_batch)):
    #         # if i == 0:
    #         #     print(y, sigmoid(y_hat))
    #         #     print(y_hat, y_int_hat)
    #         #     # exit()

    #         # data_x.append(sigmoid(y_hat))
    #         # data_y.append(y)

    #         # print(y_int_hat_batch)
    #         # print(y_int_hat)
    #         # print(y)
    #         # data_x.append(y_int_hat.item())
    #         # data_y.append(y.item())

    #         # print('correct', y)
    #         # print('True', nnue.forward(our, their).item())
    #         # print('Int', intnnue.forward(our, their)[0])

    #         z = np.polyfit(data_x, data_y, 1)
    #         p = np.poly1d(z)
    #         print(i, z, " " * 10, end='\r')

    #         if i > total_batches:
    #             break
    # except KeyboardInterrupt:
    #     pass

    # z = np.polyfit(data_x, data_y, 1)
    # p = np.poly1d(z)

    # R = np.corrcoef(data_x, data_y)
    # print('\nR =', R)

    # print(p)

    # plt.plot(data_x, data_y, 'ro')
    # plt.plot(data_x, p(data_x))
    # plt.show()
