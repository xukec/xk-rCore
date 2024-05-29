TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
#使用wildcard函数来查找APP_DIR目录下所有的.rs文件，APPS变量现在包含了一个或多个Rust源代码文件的路径。
APPS := $(wildcard $(APP_DIR)/*.rs)
#将APPS中的每个.rs文件路径（以APP_DIR为前缀）替换为以TARGET_DIR为前缀、文件名部分相同但去除了.rs扩展名的路径。这样，ELFS变量就包含了与APPS中每个.rs文件对应的ELF文件的预期路径。
#APPS = src/bin/00hello_world.rs ,APP_DIR = src/bin ,TARGET_DIR = target/riscv64gc-unknown-none-elf/release
#意思是src/bin/00hello_world.rs ==> target/riscv64gc-unknown-none-elf/release/00hello_world
#替换了路径也替换了文件后缀
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
#与ELFS的定义类似，但生成的是.bin文件的路径列表。
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

#处理RISC-V 64位架构的二进制文件
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
#@控制正在执行命令回显，加@不显示
elf:
	@python3 build.py

#对于$(ELFS)列表中的每个ELF文件路径（存储在$(elf)中），使用rust-objcopy命令将其转换为二进制文件，
#并将输出保存在与原始ELF文件相同的目录（由$(TARGET_DIR)指定）中，但文件扩展名更改为.bin。所有这一切都是在静默模式下完成的（由于@字符）。
#意思是 每个elf <==> $(elf) = target/riscv64gc-unknown-none-elf/release/00hello_world
#遍历$(ELFS) 执行命令：rust-objcopy target/riscv64gc-unknown-none-elf/release/xxx --strip-all -O binary target/riscv64gc-unknown-none-elf/release/xxx.bin
binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean #伪目标：防止与实际文件名或目录名冲突