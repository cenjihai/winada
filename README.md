# winada Windows 获取本机网络适配器信息的封装


---

封装基于 winapi 中GetAdaptersAddresses 函数。获取所有的适配器信息

````rust
pub struct AdapterInfo {
    pub adapter_name: String,   //适配器名字
    pub friendly_name: String,  //适配器友好名字，如WLAN、以太网
    pub description: String,    //适配器描述
    pub state: u32,             //适配器状态 1 开启 2关闭
    pub in_addr: Vec<String>,   // ipv4 地址列表
    pub in6_addr: Vec<String>,  //ipv6地址列表
    pub in_gw_addr: Vec<String>,    //ipv4 网关地址
    pub in6_gw_addr: Vec<String>,   //ipv6 网关地址
    pub in_dns_addr: Vec<String>,   //ipv4 dns服务器
    pub in6_dns_addr: Vec<String>,  //ipv6 dns服务器
    pub ip_mask: String,            //ipv4 的子网掩码
}
````

## 使用方法如下：

```rust
extern crate winada;

use winada::*;

fn main() {

    let all_adapter_info : Vec<AdapterInfo> = unsafe { get_all_adapter_info() };

    let iter = all_adapter_info.iter();
    for adapter_info in iter {
        println!("Adapter info: {:?}", adapter_info);
    }
}
```

author: cenjihai

Github: github/cenjihai