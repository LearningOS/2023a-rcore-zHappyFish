1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

### 编程作业

    本实验要求实现一个能查询当前正在执行的任务信息的系统调用，任务信息包括任务控制块相关信息（任务状态）、任务使用的系统调用及调用次数、任务总运行时长。
    
    对于任务状态，`TaskControlBlock` 中已经有记录，之间取出来即可，而对于任务使用的系统调用及调用次数和任务总运行时长，我在添加 `TaskControlBlock` 中添加了两个成员，`syscall_times` 和 `start_time`, `syscall_times` 的类型和 `TaskInfo` 中的一致，`start_time` 是一个 `u64` 类型。

    当一个任务第一次开始运行时，我记录它的 `start_time`，每次使用系统调用之前调整 `syscall_times` 的值即可。


### 简答作业

1.  
    RustSBI version 0.3.0-alpha.2 执行结果如下:
    ```
    [ERROR] [kernel] .bss [0x80263000, 0x8028c000)
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] Panicked at src/trap/mod.rs:73 Unsupported trap Exception(LoadFault), stval = 0x18!
    ```

2.   
    L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

        刚进入 `__restore` 时，`a0` 代表了内核栈的栈顶地址，`__restore` 可以用于返回用户空间之前的恢复上下文工作以及切换task的工作（载入被`__switch`更换后的 `TaskContext`）。
        
    L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
    ```
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    ```

        这里恢复了sstatus、sepc和scratch这三个寄存器
        sstatus：SPP 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息
        sepc：当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址
        scratch：此时保存了用户栈的栈顶地址

    L50-L56：为何跳过了 x2 和 x4？
    ```
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ```

        x2就是sp，由于此时还需要使用sp，所以暂时不恢复，x4是tp，应用不会使用它，所以无需恢复
    
    L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

    csrrw sp, sscratch, sp


        该指令后，sp是用户栈栈顶，sscratch是内核栈栈顶

    `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

        状态切换在sret指令，在S态执行该指令后会自动切换到U态，并跳转到sepc处继续执行
    
    L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
    
    csrrw sp, sscratch, sp

        该指令后，sp是内核栈顶，sscratch是用户栈栈顶
    
    从 U 态进入 S 态是哪一条指令发生的？

        ecall






    

