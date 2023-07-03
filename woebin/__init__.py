import ctypes


dll = ctypes.CDLL("./target/release/woebin.dll")

dll.wbp_new.restype = ctypes.c_void_p

dll.wbp_is_done.argtypes = [ctypes.c_void_p]
dll.wbp_is_done.restype = ctypes.c_bool

dll.wbp_get_bins_num.argtypes = [ctypes.c_void_p]


class WoeBinningProc:
    def __init__(self, desirable_bins_num):
        self.wbp = dll.wbp_new(desirable_bins_num)

    def is_done(self):
        return dll.wbp_is_done(self.wbp)

    def process_categorial(self, series, target):
        assert len(series) == len(target)
        size = len(series)

        dll.wbp_process_categorial.argtypes = [
            ctypes.c_void_p, ctypes.c_uint64, 
            (ctypes.c_uint64 * size), (ctypes.c_bool * size)
        ]
        dll.wbp_process_categorial(
            ctypes.c_void_p(self.wbp),
            size,
            (ctypes.c_uint64 * size)(*series), 
            (ctypes.c_bool * size)(*target),
        )

    def process_numeric(self, series, target):
        assert len(series) == len(target)
        size = len(series)

        dll.wbp_process_numeric.argtypes = [
            ctypes.c_void_p, ctypes.c_uint64, 
            (ctypes.c_uint64 * size), (ctypes.c_bool * size)
        ]
        dll.wbp_process_numeric(
            ctypes.c_void_p(self.wbp),
            size,
            (ctypes.c_uint64 * size)(*series), 
            (ctypes.c_bool * size)(*target),
        )

    def get_bins_info(self):
        assert self.is_done()

        # Get number of bins
        bins_num = dll.wbp_get_bins_num(self.wbp)

        # Get general information about bins
        dll.wbp_get_bins_info.argtypes = [
            ctypes.c_void_p, ctypes.c_uint64, (BinInfo * bins_num)
        ]
        bins_info = (BinInfo * bins_num)()
        dll.wbp_get_bins_info(self.wbp, bins_num, bins_info)

        bins_info_list = []

        # Extract values for each bin
        for idx, bin_info in enumerate(bins_info):
            dll.wbp_get_bin_values.argtypes = [
                ctypes.c_void_p, ctypes.c_uint64, ctypes.c_uint64, 
                (ctypes.c_uint64 * bin_info.size)
            ]
            bin_values = (ctypes.c_uint64 * bin_info.size)()
            dll.wbp_get_bin_values(self.wbp, idx, bin_info.size, bin_values)
            bins_info_list.append({
                'woe': bin_info.woe,
                'iv': bin_info.iv,
                'values': sorted(bin_values),
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


class BinInfo(ctypes.Structure):
    _fields_ = [
        ('woe', ctypes.c_double),
        ('iv', ctypes.c_double),
        ('size', ctypes.c_uint64),
    ]

    def __repr__(self):
        return f"BinInfo(size={self.size}, woe={self.woe}, iv={self.iv})"
