import struct
import math
import numpy as np
from scipy.optimize import curve_fit


SPC_DATA_FORMAT = '<' + (64 * 'H') + 'di'
HALF_KP_DATA_FORMAT = '<' + (60 * 'H') + 'di'

BATCH_SIZE = 8192
NUM_WORKERS = 24
EPOCHS = 50

_FEATURE_TYPES = {
    'SPC': ['SPC', 768],
    'HALF_KP': ['HalfKP', 40960]
}

(FEATURE_SET, NUM_FEATURES, DATA_FORMAT) = _FEATURE_TYPES['HALF_KP']

DATA_SIZE = struct.calcsize(DATA_FORMAT)

M = 32
K = 32

LR = 1e-3
LAMBDA = 1

WEIGHT_SCALE = 64
ACTIVATION_RANGE = 127

MIN = -128 / WEIGHT_SCALE
MAX = 127 / WEIGHT_SCALE


def sigmoid(x):
    if x < -709:
        return 0

    return 1 / (1 + math.exp(-x))


def get_data():
    with open('../games/val_training_data.bin', 'rb') as f:
        raw_data = f.read()

    data = {}

    for i in range(0, len(raw_data) // DATA_SIZE, 101):
        content = struct.unpack_from(DATA_FORMAT, raw_data, i*DATA_SIZE)
        final_score = content[64]
        score = content[65]

        entry = data.get(score, {
            'num_games': 0,
            'total_score': 0
        })

        entry['num_games'] += 1
        entry['total_score'] += final_score

        data[score] = entry

    data = [[k, entry['total_score'] / entry['num_games']]
            for (k, entry) in data.items()]

    data.sort(key=lambda xy: xy[0])

    return list(zip(*data))


def calc_scaling():
    def np_sigmoid(x, k):
        clipped = np.clip(-x / k, -np.inf, 709)
        return 1 / (1 + np.exp(clipped))

    (x, y) = get_data()
    first_guess = 100

    popt, pcov = curve_fit(np_sigmoid, x, y, first_guess, method='dogbox')
    return float(popt[0])


# SCALING_FACTOR = calc_scaling()
