import sys
import os
import ctypes
import math

import sys
import os

from .version import __version__


# Define NoneType
NoneType = type(None)


# Allowed types in series
ALLOWED_SERIES_TYPES = [str, float, int, bool, NoneType]

DEV_DLL_DIR = "./target/release"


def get_dll_path():
    dll_name = None

    if sys.platform == "linux" or sys.platform == "linux2":
        dll_name = 'libwoebin.so'
    elif sys.platform == "darwin":
        dll_name = 'libwoebin.dylib'
    elif sys.platform == "win32":
        dll_name = 'woebin.dll'

    assert dll_name is not None, f"OS not supported: {sys.platform}"

    dev_dll_path = os.path.join(DEV_DLL_DIR, dll_name)

    if os.path.exists(dev_dll_path):
        return dev_dll_path
    else:
        return os.path.join(sys.prefix, 'dlls', dll_name)


def load_dll(dll_path):
    dll = ctypes.CDLL(dll_path)

    dll.wbp_new.argtypes = [ctypes.c_uint64, ctypes.c_double]
    dll.wbp_new.restype = ctypes.c_void_p
    dll.wbp_is_done.argtypes = [ctypes.c_void_p]
    dll.wbp_is_done.restype = ctypes.c_bool
    dll.wbp_get_bins_num.argtypes = [ctypes.c_void_p]

    return dll


dll_path = get_dll_path()
dll = load_dll(dll_path)


class WoeBinningProc:
    def __init__(self):
        self._wbp = None
        self._value_map = None
        self._value_map_back = None

    def reset(self):
        self._wbp = None
        self._value_map = {}
        self._value_map_back = {}

    def process(self, series, target, bins=10, is_numeric=False, smooth=1.0):
        assert len(series) == len(target)

        size = len(series)

        target = map(bool, target)

        self.reset()

        series_hashed = self._preproc_series(series, is_numeric)

        size = len(series)
        self._wbp = dll.wbp_new(bins, smooth)

        process_method = dll.wbp_process_numeric if is_numeric \
            else dll.wbp_process_categorial

        process_method(
            ctypes.c_void_p(self._wbp),
            ctypes.c_uint64(size),
            (ctypes.c_uint64 * size)(*series_hashed), 
            (ctypes.c_bool * size)(*target),
        )

    def get_bins_info(self):
        # Get number of bins
        bins_num = dll.wbp_get_bins_num(self._wbp)

        # Get general information about bins
        bins_info = (BinInfo * bins_num)()
        dll.wbp_get_bins_info(
            ctypes.c_void_p(self._wbp), 
            ctypes.c_uint64(bins_num), 
            bins_info
        )

        bins_info_list = []

        # Extract values for each bin
        for idx, bin_info in enumerate(bins_info):
            bin_values_hashed = (ctypes.c_uint64 * bin_info.size)()
            dll.wbp_get_bin_values(
                ctypes.c_void_p(self._wbp), 
                ctypes.c_uint64(idx), 
                ctypes.c_uint64(bin_info.size), 
                bin_values_hashed
            )
            bin_values = list(map(
                lambda val: self._value_map.get(val, val),
                bin_values_hashed
            ))
            bins_info_list.append({
                'woe': bin_info.woe,
                'iv': bin_info.iv,
                'values': bin_values,
            })

        return bins_info_list

    def get_woe_map(self):
        woe_map = {}

        for info in self.get_bins_info():
            for value in info['values']:
                woe_map[value] = info['woe']

        return woe_map

    def get_iv_total(self):
        iv_total = 0.0
        for info in self.get_bins_info():
            iv_total += info['iv']
        return iv_total

    def _preproc_series(self, series, is_numeric):
        series_type = detect_series_type(series, raise_on_not_numeric=is_numeric)
        series_hashed = self._convert_series(series, series_type)
        return series_hashed

    def _convert_series(self, series, series_type):
        none_hash = unsigned_hash(None)

        if series_type is int:
            self._value_map = {
                none_hash: None,
            }
            self._value_map_back = {
                None: none_hash,
            }
            converted = list(map(
                lambda e: self._value_map_back.get(e, int(e)),
                series
            ))

        elif series_type is bool:
            self._value_map = {
                0: False,
                1: True,
                none_hash: None,
            }
            self._value_map_back = {
                False: 0,
                True: 1,
                None: none_hash,
            }
            converted = list(map(
                lambda e: self._value_map_back.get(e, int(e)),
                series
            ))

        elif series_type is str:
            converted = list(map(
                lambda x: unsigned_hash(str(x)) 
                    if not (isinstance(x, float) and math.isnan(x)) and x is not None 
                    else none_hash,
                series
            ))
            self._value_map = dict(zip(converted, series))
            self._value_map_back = dict(zip(series, converted))

        elif series_type is float:
            unique = sorted(set(drop_nan(series)))
            self._value_map = dict(zip(range(1, 1 + len(unique)), unique))
            self._value_map[none_hash] = None
            self._value_map_back = dict(
                reversed(pair) for pair in self._value_map.items()
            )
            converted = list(map(
                lambda e: self._value_map_back.get(e, none_hash),
                series
            ))

        else:
            raise TypeError(f"Type not supported: {series_type}")

        return converted


def detect_series_type(series, raise_on_not_numeric=False):
    # Get unique types
    types = set(map(type, series))

    # Check numeric
    if raise_on_not_numeric:
        if str in types:
            raise TypeError(f"{str} is not numeric")
        if NoneType in types:
            raise TypeError(f"{NoneType} is not numeric")
        if any(math.isnan(e) for e in series):
            raise TypeError(f"{math.nan} is not numeric")

    # Check for complex types
    if types - set(ALLOWED_SERIES_TYPES):
        raise TypeError("Complex data type in series")

    # Return the most general type
    for type_ in ALLOWED_SERIES_TYPES:
        if type_ in types:
            return type_

    raise TypeError("No allowed data types detected")


def drop_nan(series):
    return list(filter(lambda e: not math.isnan(e) and e is not None, series))


def unsigned_hash(x):
    return ctypes.c_uint64(hash(x)).value


class BinInfo(ctypes.Structure):
    _fields_ = [
        ('woe', ctypes.c_double),
        ('iv', ctypes.c_double),
        ('size', ctypes.c_uint64),
    ]

    def __repr__(self):
        return f"BinInfo(size={self.size}, woe={self.woe}, iv={self.iv})"
