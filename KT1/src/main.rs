fn main() {
  const LENGTH: i8 = 100;  
  let mut arr: [i128; 3] = [0, 1, 1];

  println!("Here are {} first numbers of the fibonacci sequence:", LENGTH);

  println!("1: 0");
  println!("2: 1");
  
  for i in 2..LENGTH{
    println!("{}: {}",i+1,arr[2]);
    arr[0]=arr[1];
    arr[1]=arr[2];
    arr[2]=arr[0]+arr[1]
  }
}
