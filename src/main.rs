use std::fs;
use elf;
use rand::Rng;
use std::io::{Seek, SeekFrom, Write};

fn main() {
    let mut args:Vec<String> =std::env::args().collect();

    let current_path= std::env::current_exe().unwrap();
    let path:String= current_path.parent().unwrap().to_string_lossy().into();
    if args.len()!=2{
        println!("br [so path]");
        return;
    }

    println!("load names.txt");
    let mut names_dict=vec![];
    for p in fs::read_to_string(path+"/names.txt").unwrap().split("\n"){
       names_dict.push(p.trim().to_string());
    }

    let path = args.remove(1);
    let (offset,_,buff)= {
        println!("load {}", path);
        let elf_file = elf::File::open_path(&path).unwrap();
        let ro_data = elf_file.get_section(".rodata").unwrap();
        println!("{:?}", ro_data.shdr);
        (ro_data.shdr.addr, ro_data.shdr.size, ro_data.data.clone())
    };

    let names=get_strings(offset as usize,&buff);
    let mut write_table=vec![];
    for name in names.iter() {
        for p in names_dict.iter() {
            if let Some(index)=name.context.find(&p[..]){
               let c=name.context.as_bytes()[index-1];
               if c >=48 && c<=57 {
                   let mut data = Vec::with_capacity(name.len);
                   for _ in 0..name.len {
                       data.push(rand::thread_rng().gen_range(1..255))
                   }
                   println!("{}->{:?}", name.context, data);
                   write_table.push(WriteStr {
                       start: name.offset,
                       size: name.len,
                       data
                   });

                   continue;
               }
            }
        }
    }

    if write_table.len()==0{
        println!("no found info");
        return;
    }
    if let Err(err)= fs::copy(&path,path.clone()+".bak"){
        println!("bak file {} fail:{}",path,err);
    }

    let mut sofile=  fs::OpenOptions::new().write(true).open(path).unwrap();
    for w in write_table {
        sofile.seek(SeekFrom::Start(w.start as u64)).unwrap();
        sofile.write(&w.data).unwrap();
    }
    println!("Close");

}

#[derive(Debug)]
struct WriteStr{
    start:usize,
    size:usize,
    data:Vec<u8>
}

#[derive(Debug)]
struct Name{
    offset:usize,
    len:usize,
    context:String
}

fn get_strings(start_offset:usize, buff:&Vec<u8>)->Vec<Name>{
    let mut r=vec![];
    let mut start: usize=0;
    for (i,p) in buff.iter().enumerate() {
        if *p==0u8 {
            match String::from_utf8(buff[start..i].to_vec()) {
                Ok(str) => {
                    if str.trim() !="" {
                        r.push(Name {
                            offset:start_offset+ start,
                            len: i - start,
                            context: str
                        });
                    }
                }
                Err(_) => {}
            }
            start = i + 1;
        }
    }
    r
}