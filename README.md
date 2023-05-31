# CHIP-8 Emülatörü

Bu, Rust programlama dili kullanılarak CHIP-8 sanal işlemcisinin basit bir emülatörünü içeren bir projedir. CHIP-8, 1970'ler ve 1980'lerde popüler olan eski bir oyun konsolu ve programlama dilidir.

## Başlangıç

1. Bu projeyi yerel bilgisayarınıza kopyalayın: 

```
git clone https://github.com/alihan-tadal/chip8-emulator.git
```

2. Rust programlama dilini yükleyin: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

3. Projeyi aşağıdaki komutla derleyin ve çalıştırın:

```
cd chip8-emulator
cargo run
```

## Emülatörün Kullanımı

1. Emülatör çalıştığında, CHIP-8 programını yüklemek için bellek alanında (memory) değişiklikler yapabilirsiniz. Örnek bir program örneği mevcuttur.

2. Emülatör, CHIP-8'in temel 35 adet opcode'unu destekler. İşlemcideki (registers) kaydedicileri güncelleyebilir ve opcode'ları çalıştırabilirsiniz.

3. Emülatörün çalışması sırasında, emülatörün çıktısı opcode'ları ve işlem sonuçlarını gösterir.
