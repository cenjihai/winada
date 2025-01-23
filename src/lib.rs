mod windows;

pub use windows::{AdapterInfo,get_all_adapter_info};

#[cfg(test)]
mod tests {
    use crate::windows::{ get_all_adapter_info, AdapterInfo};
    #[test]
    fn it_works() {
        unsafe {
            let all_adapter: Vec<AdapterInfo> = get_all_adapter_info();
            let iter = all_adapter.iter();
            for adapter in iter {
                println!("{:?}", adapter.adapter_name);
            }
        }
    }
}
