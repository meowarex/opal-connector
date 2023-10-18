#[cfg(target_family = "unix")]
use sysinfo::{ProcessorExt, SystemExt};

use crate::types::CPUStats;

use anyhow::Result;

#[cfg(target_family = "windows")]
use windows::Win32::System::Performance::*;

#[cfg(target_family = "windows")]
use std::ptr;

#[cfg(target_family = "windows")]
use windows::Win32::Foundation::ERROR_SUCCESS;

#[cfg(target_family = "windows")]
use std::ffi::CStr;

#[cfg(target_family = "windows")]
use std::mem;

#[cfg(target_family = "windows")]
use std::process::exit;

use super::DataCollector;

#[cfg(target_family = "unix")]
impl DataCollector {
  /// Gets the current CPU stats
  /// wait what the fuck this is an array of cores? 🥴👍
  pub fn get_cpu(&mut self) -> Result<CPUStats> {
    let (mut usage, mut freq) = (vec![], vec![]);

    for processor in self.fetcher.processors() {
      usage.push(processor.cpu_usage().floor() as u16);
      freq.push(processor.frequency() as u16);
    }

    self.fetcher.refresh_cpu();

    Ok(CPUStats { usage, freq })
  }
}

#[cfg(target_family = "windows")]
impl DataCollector {
  pub fn get_cpu(&mut self) -> Result<CPUStats> {
    let (mut usage, mut freq) = (vec![], vec![]);

    // Stores information about the capacity of the buffer needed for retrieving
    // counter data from PDH counter methods.
    let mut proc_freq_sz: u32 = 0;
    let mut proc_freq_cnt: u32 = 0;
    let mut proc_perf_sz: u32 = 0;
    let mut proc_perf_cnt: u32 = 0;
    let mut proc_util_sz: u32 = 0;
    let mut proc_util_cnt: u32 = 0;

    // Pointers we need to pass to the aforementioned PDH library methods so that
    // it can update the values stored here for us to allocate the buffers right.
    let ptr_freq_sz: *mut u32 = &mut proc_freq_sz;
    let ptr_freq_cnt: *mut u32 = &mut proc_freq_cnt;
    let ptr_perf_sz: *mut u32 = &mut proc_perf_sz;
    let ptr_perf_cnt: *mut u32 = &mut proc_perf_cnt;
    let ptr_util_sz: *mut u32 = &mut proc_util_sz;
    let ptr_util_cnt: *mut u32 = &mut proc_util_cnt;

    // Our target for retrieving frequency and usage data. This can either be the
    // data we retrieved during this method invocation or reusing cached data due to
    // counter data rolling over during that particular fetch.
    let ptr_freq_data_target: *mut PDH_FMT_COUNTERVALUE_ITEM_A;
    let ptr_perf_data_target: *mut PDH_FMT_COUNTERVALUE_ITEM_A;
    let ptr_util_data_target: *mut PDH_FMT_COUNTERVALUE_ITEM_A;

    unsafe {
      // Confirm all counters are ready to provide data first as well as
      // the number of items being returned and the size of the buffer the
      // PDH library wants from us.
      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_freq_counter,
                                             PDH_FMT_DOUBLE,
                                             ptr_freq_sz,
                                             ptr_freq_cnt,
                                             ptr::null_mut());

      // This return value is given specifically because we pass a null pointer for
      // the buffer, indicating more room is needed to write. Obviously, absent
      // an actual buffer, it can't write anything. Any status other than this is
      // indicative of a problem.
      if ret != PDH_MORE_DATA {
        eprintln!("Unable to fetch processor frequency data. Errno: {}", ret);
        exit(ret);
      }

      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_perf_counter,
                                             PDH_FMT_DOUBLE,
                                             ptr_perf_sz,
                                             ptr_perf_cnt,
                                             ptr::null_mut());

      if ret != PDH_MORE_DATA {
        eprintln!("Unable to fetch processor performance data. Errno: {}", ret);
        exit(ret);
      }

      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_util_counter,
                                             PDH_FMT_DOUBLE,
                                             ptr_util_sz,
                                             ptr_util_cnt,
                                             ptr::null_mut());

      if ret != PDH_MORE_DATA {
        eprintln!("Unable to fetch processor utilization data. Errno: {}", ret);
        exit(ret);
      }

      // The windows-rs crate does a little trolling and fails to generate an array of
      // counter value item structs whose size matches the output the above calls
      // are given. This could be due to mismatch in how the rust struct implementation
      // is handled vs what the C struct is like. It could be a byte alignment issue.
      // We simply do not know.
      //
      // However, this method *does* continue to work given some padding to avoid
      // buffer overruns. So double the size of our allocation to capture
      // this problem and then proceed not to worry about it for the foreseeable future.
      let mut proc_freq_data: Vec<PDH_FMT_COUNTERVALUE_ITEM_A> =
          vec!(PDH_FMT_COUNTERVALUE_ITEM_A::default(); (proc_freq_cnt * 2) as usize);

      let mut proc_perf_data: Vec<PDH_FMT_COUNTERVALUE_ITEM_A> =
          vec!(PDH_FMT_COUNTERVALUE_ITEM_A::default(); (proc_perf_cnt * 2) as usize);

      let mut proc_util_data: Vec<PDH_FMT_COUNTERVALUE_ITEM_A> =
          vec!(PDH_FMT_COUNTERVALUE_ITEM_A::default(); (proc_util_cnt * 2) as usize);

      // Now when we invoke these methods again they should always return ERROR_SUCCESS.
      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_freq_counter,
                                   PDH_FMT_DOUBLE,
                                   ptr_freq_sz,
                                   ptr_freq_cnt,
                                   proc_freq_data.as_mut_ptr());

      if ret != ERROR_SUCCESS.0 as i32 {
          eprintln!("Unable to fetch processor frequency data. Errno: {}", ret);
          exit(ret);
      }


      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_perf_counter,
                                   PDH_FMT_DOUBLE,
                                   ptr_perf_sz,
                                   ptr_perf_cnt,
                                   proc_perf_data.as_mut_ptr());

      if ret != ERROR_SUCCESS.0 as i32 {
          eprintln!("Unable to fetch processor performance data. Errno: {}", ret);
          exit(ret);
      }


      let ret = PdhGetFormattedCounterArrayA(self.pdh_proc_util_counter,
                                                  PDH_FMT_DOUBLE,
                                                  ptr_util_sz,
                                                  ptr_util_cnt,
                                                  proc_util_data.as_mut_ptr());

      if ret != ERROR_SUCCESS.0 as i32 {
          eprintln!("Unable to fetch processor utilization data. Errno: {}", ret);
          exit(ret);
      }


      if (proc_freq_cnt == 0 || proc_perf_cnt == 0 || proc_util_cnt == 0) ||
          ((proc_freq_cnt != proc_perf_cnt) || (proc_freq_cnt != proc_util_cnt)) {
        // Sometimes one or more counters will roll over (integer overflow) numerically
        // and so the difference between the most recent counter poll and the previous
        // one will not pass the sniff test, and the pdh library will flip a tit about it.
        // In this instance we should just repeat the last data that was valid and then
        // try again on the next poll. In which case the issue will resolve itself.

        if self.pdh_proc_freq_data_len == 0 {
            eprintln!("Unable to fetch data from counters and nothing is cached.");
            exit(1);
        }
        ptr_freq_data_target = self.pdh_proc_freq_data_cached;
        ptr_perf_data_target = self.pdh_proc_perf_data_cached;
        ptr_util_data_target = self.pdh_proc_util_data_cached;
      } else {

        // Data received checks out -- continue and replace cached values.
        ptr_freq_data_target = proc_freq_data.as_mut_ptr();
        ptr_perf_data_target = proc_perf_data.as_mut_ptr();
        ptr_util_data_target = proc_util_data.as_mut_ptr();

        // Reassemble cached data as vector so that Rust can deallocate it. Then drop it.
        // Do not attempt if there is nothing in the cache already.
        if self.pdh_proc_freq_data_len != 0 {
            let proc_freq_data_old = Vec::from_raw_parts(self.pdh_proc_freq_data_cached,
                                                                                      self.pdh_proc_freq_data_len,
                                                                                             self.pdh_proc_freq_data_capacity);
            drop(proc_freq_data_old);



            let proc_perf_data_old = Vec::from_raw_parts(self.pdh_proc_perf_data_cached,
                                                                                        self.pdh_proc_perf_data_len,
                                                                                               self.pdh_proc_perf_data_capacity);
            drop(proc_perf_data_old);

            let proc_util_data_old = Vec::from_raw_parts(self.pdh_proc_util_data_cached,
                                                                                      self.pdh_proc_util_data_len,
                                                                                             self.pdh_proc_util_data_capacity);
            drop(proc_util_data_old);
        }

        self.pdh_proc_freq_data_cached = proc_freq_data.as_mut_ptr();
        self.pdh_proc_freq_data_len = proc_freq_data.len();
        self.pdh_proc_freq_data_capacity = proc_freq_data.capacity();

        self.pdh_proc_perf_data_cached = proc_perf_data.as_mut_ptr();
        self.pdh_proc_perf_data_len = proc_perf_data.len();
        self.pdh_proc_perf_data_capacity = proc_perf_data.capacity();

        self.pdh_proc_util_data_cached = proc_util_data.as_mut_ptr();
        self.pdh_proc_util_data_len = proc_util_data.len();
        self.pdh_proc_util_data_capacity = proc_util_data.capacity();

        // Prevent Rust from immediately deallocating vec contents at end of method invocation.
        // Otherwise our cached values will go bye bye.
        mem::forget(proc_freq_data);
        mem::forget(proc_perf_data);
        mem::forget(proc_util_data);
      }

        let mut i = 0 as isize;

        while i < proc_freq_cnt as isize {
            let sz_name_cstr: &CStr = CStr::from_ptr(
                (*ptr_freq_data_target.offset(i)).szName.0 as *const i8);
            let str_slice: &str = sz_name_cstr.to_str().unwrap_or("oh shit");
            if str_slice.contains("Total") {
                i = i + 1;
                continue;
            } else if str_slice.contains("oh shit") {
                eprintln!("Inconsistent data received from perfmon. Aborting.");
                exit(1);
            }

            usage.push(f64::floor(
                (*ptr_util_data_target.offset(i)).FmtValue.Anonymous.doubleValue) as u16);
            freq.push(f64::floor(
                (*ptr_freq_data_target.offset(i)).FmtValue.Anonymous.doubleValue *
                     ((*ptr_perf_data_target.offset(i)).FmtValue.Anonymous.doubleValue / 100.0)) as u16);

            i = i + 1;
        }
    }

    Ok(CPUStats { usage, freq })
  }
}