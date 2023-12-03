
use regex::Regex;
use winapi::shared::minwindef::{DWORD, HKEY};
use winapi::um::winnt::LPCSTR;
use std::char::from_u32;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::process::Command;
use std::arch::asm;
use std::ptr::null_mut;
use std::str::from_utf8;
use std::{error, mem};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot,TH32CS_SNAPPROCESS,PROCESSENTRY32, Process32First, Process32Next};
use winapi::um::winreg::{HKEY_CLASSES_ROOT,RegOpenKeyA};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use wmi::{Variant, COMLibrary};
use std::path::Path;

fn DiskDriveCheck()->i8{
   let wmi_con = wmi::WMIConnection::new(COMLibrary::new().unwrap()).unwrap();
   let results: Vec<HashMap<String, String>> = wmi_con.raw_query("select CAption from win32_DiskDrive").unwrap();
   if results.len()>0{
      let caption=&results[0]["Caption"];
      if caption.find("VMware")!=None{
         println!("主板信息检测为VMware");
         return 1;
      }
   }
   println!("DiskDrive检测通过");
   return 0;
}

//MAC地址检测
fn GetMacAddress() -> i8{
   let strMacList=["00-0C-29-92-EB","00-05-69","00-0c-29","00-50-56"];
   let ipconfig=Command::new("ipconfig").arg("/all").output().unwrap();
   let output=String::from_utf8_lossy(&ipconfig.stdout);
   let re=Regex::new("[A-Z-0-9]{2}[-][A-Z-0-9]{2}[-][A-Z-0-9]{2}[-][A-Z-0-9]{2}[-][A-Z-0-9]{2}[-][A-Z-0-9]{2}").unwrap();
   for(i,c) in re.captures_iter(&output).enumerate(){
      for j in 0..c.len(){
       let mac_address=&c[j];
       for macaddr in strMacList{
         if(mac_address.find(macaddr)!=None){
            println!("Mac地址检测为虚拟机，特征:{:#?}",macaddr);
            return 1;
          }
       }
      }
   }
   println!("Mac地址检测通过");
   return 0;
}

//CPUDID检测
fn Getcpuid() -> i8{
   let mut c:u32=0;
   unsafe{
      asm!(
         "mov eax,1",
         "cpuid",
         "and ecx, 0x80000000",
         lateout("ecx") c,
      )
   }
   let vm=c != 0;
   if vm{
      println!("CPUID检测为虚拟机");
      return 1;
   }
   println!("CPUID检测通过");
   return 0;
}

fn ProcessCheck()->i8{
   unsafe { 
      let mut entry: PROCESSENTRY32 = mem::zeroed();
      entry.dwSize = mem::size_of::<PROCESSENTRY32>() as DWORD;
      let hProcessSnap=CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
      if hProcessSnap==INVALID_HANDLE_VALUE{
         println!("获取映像列表失败");
         return 1;
      }

      let mut bMore=Process32First(hProcessSnap,&mut entry);
      let mut bm_=bMore!=0;
      if bMore==0{
         println!("获取进程列表失败");
         return 1;
      }

      while bm_ {
         let mut processName=entry.szExeFile;
         let slice=CStr::from_ptr(processName.as_ptr());
         let proc_name_str = slice.to_str().unwrap();
         if proc_name_str=="vmtoolsd.exe"{
            println!("进程检测到:vmtoolsd.exe");
            return 1;
         }
         bMore=Process32Next(hProcessSnap, &mut entry);
         if bMore==0{
            bm_=bMore!=0;
            break;
         }
      }
   };
   println!("进程检测通过");
   return 0;
}

fn regeditCheck()->i8{
   let mut hkey = null_mut();
   let path=CString::new("\\Applications\\VMwareHostOpen.exe").unwrap();
   let rquery={ unsafe { RegOpenKeyA(HKEY_CLASSES_ROOT,path.as_ptr() , &mut hkey) } };
   let result=rquery != 0;
   if result{
      println!("注册表检测通过");
   }else{
      println!("注册表检测未通过，特征HKCU\\Applications\\VMwareHostOpen.exe");
   }
   return 0;
}


fn serviceCheck() -> i8{
   let mut CaptionName="Virtual Disk";
   let wmi_con = wmi::WMIConnection::new(COMLibrary::new().unwrap()).unwrap();
   let results: Vec<HashMap<String, String>> = wmi_con.raw_query("SELECT Caption FROM Win32_Service where Caption=\"Virtual Disk\"").unwrap();
   if results.len()>0{
      let caption=&results[0]["Caption"];
      if caption.find(CaptionName)!=None{
         println!("服务检测到:{:?}",CaptionName);
         return 1;
      }
   }else{
      println!("服务检测通过");
   }

   return 0;

}

fn PathCheck()->i8{
   let mut Path=Path::new("C:\\Program Files\\VMware\\");
   if Path.exists()==true{
      println!("C:\\Program Files存在VMware");
      return 1;
   }else{
      println!("路径检测通过");
   }
   return 0;
}


fn main() {
   DiskDriveCheck();
   GetMacAddress();
   Getcpuid();
   ProcessCheck();
   regeditCheck();
   serviceCheck();
   PathCheck();
}