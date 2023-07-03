use crate::woe_binning_proc::WoeBinningProc;


#[repr(C)]
struct BinInfo {
    woe: f64,
    iv: f64,
    size: usize,
}


struct WoeBinningProcWrapper {
    wbp: WoeBinningProc,
}


impl WoeBinningProcWrapper {
    #[no_mangle]
    #[export_name="wbp_new"]
    pub extern fn new(desirable_bins_num: usize) -> Box<Self> {
        Box::new(Self {
            wbp: WoeBinningProc::new(desirable_bins_num)
        })
    }

    #[no_mangle]
    #[export_name="wbp_process_categorial"]
    pub extern fn process_categorial(&mut self, size: usize, series: *const usize, 
                                     target: *const bool) {
        let (series_buff, target_buff) = unsafe {
            Self::_prepare_dataset(size, series, target)
        };
        self.wbp.process_categorial(&series_buff, &target_buff);
    }

    #[no_mangle]
    #[export_name="wbp_process_numeric"]
    pub extern fn process_numeric(&mut self, size: usize, series: *const usize, 
                                  target: *const bool) {
        let (series_buff, target_buff) = unsafe {
            Self::_prepare_dataset(size, series, target)
        };
        self.wbp.process_numeric(&series_buff, &target_buff);
    }

    #[no_mangle]
    #[export_name="wbp_is_done"]
    pub extern fn is_done(&self) -> bool {
        self.wbp.is_done()
    }

    #[no_mangle]
    #[export_name="wbp_get_bins_num"]
    pub extern fn get_bins_num(&self) -> usize {
        self.wbp.get_bins_num().unwrap()
    }

    #[no_mangle]
    #[export_name="wbp_get_bins_info"]
    pub extern fn get_bins_info(&self, bins_num: usize, 
                                bin_info_array: *mut BinInfo) {
        let woe_vec = self.wbp.get_woe_array().unwrap();
        let iv_vec = self.wbp.get_iv_array().unwrap();
        let size_vec = self.wbp.get_size_array().unwrap();

        let bins_info = (0..bins_num).map(|i| BinInfo { 
            woe: woe_vec[i], iv: iv_vec[i], size: size_vec[i] 
        }).collect::<Vec<BinInfo>>();

        unsafe {
            bins_info.as_ptr().copy_to(bin_info_array, bins_num);
        }
    }

    #[no_mangle]
    #[export_name="wbp_get_bin_values"]
    pub extern fn get_bin_values(&self, bin_idx: usize, size: usize, 
                                 values_array: *mut usize) {
        let values = self.wbp.get_bin_values(bin_idx).unwrap();
        unsafe {
            values.as_ptr().copy_to(values_array, size);
        }
    }

    unsafe fn _prepare_dataset(
                size: usize, series: *const usize, target: *const bool) -> 
                (Vec<usize>, Vec<bool>) {
        let mut series_buff: Vec<usize> = vec![0; size];
        let mut target_buff: Vec<bool> = vec![false; size];

        unsafe {
            series.copy_to(series_buff.as_mut_ptr(), size);
        }
        unsafe {
            target.copy_to(target_buff.as_mut_ptr(), size);
        }

        (series_buff, target_buff)
    }
}
