import ctypes
import random

from woebin import WoeBinningProc


random.seed(0)


def example1():
    dll = ctypes.CDLL("./target/release/woebin.dll")
    print(dll)

    dll.wbp_new.restype = ctypes.c_void_p
    wbp = dll.wbp_new(5)
    print(wbp)

    dll.wbp_is_done.argtypes = [ctypes.c_void_p]
    dll.wbp_is_done.restype = ctypes.c_bool
    print(dll.wbp_is_done(wbp))

    size = 100000
    series = [random.randint(10001, 10100) for _ in range(size)]
    target = [random.choice([False, True]) for _ in range(size)]

    dll.wbp_process_categorial.argtypes = [ctypes.c_void_p, ctypes.c_uint64, (ctypes.c_uint64 * size), (ctypes.c_bool * size)]
    dll.wbp_process_categorial(
        ctypes.c_void_p(wbp),
        size,
        ((ctypes.c_uint64 * size)(*series)), 
        ((ctypes.c_bool * size)(*target)),
    )

    print(dll.wbp_is_done(ctypes.c_void_p(wbp)))

    dll.wbp_get_bins_num.argtypes = [ctypes.c_void_p]
    bins_num = dll.wbp_get_bins_num(wbp)
    print(bins_num)

    dll.wbp_get_iv_total.argtypes = [ctypes.c_void_p]
    dll.wbp_get_iv_total.restype = ctypes.c_double
    print(dll.wbp_get_iv_total(wbp))

    class BinInfo(ctypes.Structure):
        _fields_ = [
            ('woe', ctypes.c_double),
            ('iv', ctypes.c_double),
            ('size', ctypes.c_uint64),
        ]

        def __repr__(self):
            return f"BinInfo(size={self.size}, woe={self.woe}, iv={self.iv})"

    dll.wbp_get_bins_info.argtypes = [ctypes.c_void_p, ctypes.c_uint64, (BinInfo * bins_num)]
    bins_info = (BinInfo * bins_num)()
    dll.wbp_get_bins_info(wbp, bins_num, bins_info)
    print(list(bins_info))

    idx = 1
    bin_size = bins_info[idx].size

    dll.wbp_get_bin_values.argtypes = [ctypes.c_void_p, ctypes.c_uint64, ctypes.c_uint64, (ctypes.c_uint64 * bin_size)]
    bin_values = (ctypes.c_uint64 * bin_size)()
    dll.wbp_get_bin_values(wbp, idx, bin_size, bin_values)
    print(list(bin_values))


def example2():
    size = 100000
    series = [random.randint(10001, 10100) for _ in range(size)]
    target = [random.choice([False, True]) for _ in range(size)]

    wbp = WoeBinningProc(5)
    print(wbp.is_done())

    wbp.process_categorial(series, target)
    print(wbp.is_done())

    bins_info = wbp.get_bins_info()
    print(bins_info)

    woe_map = wbp.get_woe_map()
    print(woe_map)


if __name__ == "__main__":
    # example1()
    example2()
