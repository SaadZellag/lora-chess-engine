import matplotlib.pyplot as plt
import numpy as np
import math
from consts import *

SCALING_FACTOR = calc_scaling()

(x, y) = get_data()
y2 = [sigmoid(v / ACTIVATION_RANGE / WEIGHT_SCALE * 8) for v in x]
print('Calculated SCALING_FACTOR:', SCALING_FACTOR)
print('Manual SCALING_FACTOR:', ACTIVATION_RANGE * WEIGHT_SCALE / 8)


plt.plot(x, y, 'ro')
plt.plot(x, y2)

plt.xlim([min(x), max(x)])
plt.show()
