from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch

OUT = Path(__file__).parent

fig, ax = plt.subplots(figsize=(16, 12))
ax.set_xlim(0, 16)
ax.set_ylim(0, 12)
ax.axis("off")

GOLD = "#D4AF37"
BRIGHT_GOLD = "#FFD700"
DARK_BG = "#120E0A"
CARD_BG = "#1E1812"
BORDER_GOLD = "#A57D19"
TEXT_LIGHT = "#F0E4D0"
TEXT_MUTED = "#C8B496"
GREEN = "#28D278"

fig.patch.set_facecolor(DARK_BG)
ax.set_facecolor(DARK_BG)

def box(x, y, w, h, title, detail, accent=GOLD, size=10):
    patch = FancyBboxPatch(
        (x, y), w, h,
        boxstyle="round,pad=0.08",
        linewidth=1.8,
        edgecolor=accent,
        facecolor=CARD_BG
    )
    ax.add_patch(patch)
    ax.text(x + w/2, y + h*0.68, title,
            ha="center", va="center",
            fontsize=size, fontweight="bold", color=BRIGHT_GOLD)
    ax.text(x + w/2, y + h*0.32, detail,
            ha="center", va="center", fontsize=8.5, color=TEXT_LIGHT)

def arrow(x1, y1, x2, y2, color=GOLD):
    ax.add_patch(FancyArrowPatch(
        (x1, y1), (x2, y2),
        arrowstyle="-|>", mutation_scale=15,
        linewidth=1.6, color=color
    ))

ax.text(8, 11.60, "SRIKESH OS — MASTER SYSTEM ARCHITECTURE",
        ha="center", fontsize=18, fontweight="bold", color=BRIGHT_GOLD)
ax.text(8, 11.20, "x86-64 Rust Bare-Metal Microkernel (no_std) | Dual UEFI & Legacy BIOS | Oracle VirtualBox 7.2.22",
        ha="center", fontsize=10.5, color=TEXT_MUTED)

# Boot path
box(0.6, 9.65, 3.2, 1.1, "HOST TOOLCHAIN",
    "Rust Nightly 1.102 • Cargo • bindeps")
box(6.4, 9.65, 3.2, 1.1, "BOOTABLE MEDIA",
    "Hybrid BIOS & UEFI • 3.5 MB USB Image")
box(12.2, 9.65, 3.2, 1.1, "BOOTLOADER 0.11",
    "Stage 1-4 • ExitBootServices • GOP")

arrow(3.8, 10.2, 6.4, 10.2)
arrow(9.6, 10.2, 12.2, 10.2)

# Kernel entry
box(4.5, 7.95, 7.0, 1.15, "SRIKESH OS KERNEL (kernel_main)",
    "no_std • Paging CR3 • IDT Armed • Zero-Write StorageGuard Active", accent=BRIGHT_GOLD, size=11)
arrow(13.8, 9.65, 10.5, 8.5)

# Hardware and interrupt services
box(0.4, 5.95, 3.5, 1.25, "CPU & MEMORY",
    "CPUID Intel/AMD • Usable RAM 1018MB\nFrame Allocator • Bump/Free Heap")
box(4.3, 5.95, 3.5, 1.25, "HARDWARE DRIVERS",
    "Intel e1000 Gigabit • Wi-Fi Scanner\nATA Sector Persistence (LBA 2048)")
box(8.2, 5.95, 3.5, 1.25, "TASK & SCHEDULER",
    "TCB (READY/RUN/BLOCK) • 5-tick Quantum\nRound Robin Preemption (100 Hz PIT)")
box(12.1, 5.95, 3.5, 1.25, "SECURITY & DIAGNOSTICS",
    "Host Storage Zero-Write Guard Locked\nSerial COM1 Telemetry • ACPI Power")

for x in [2.15, 6.05, 9.95, 13.85]:
    arrow(8, 7.95, x, 7.2)

# Desktop Compositor & Applications
box(1.5, 3.85, 6.0, 1.35, "EGYPTIAN ROYAL COMPOSITOR",
    "Double-Buffered 32-bit Framebuffer (800x600 & 1024x768)\nSupplied Jackal Wallpaper & Pharaoh Welcome Screen", accent=BRIGHT_GOLD)
box(8.5, 3.85, 6.0, 1.35, "11 NATIVE APPLICATIONS & 9 GAMES",
    "S-Web Browser (HTTP) • S-Shell CLI • Files • Editor • Calc\n9 Games: Chess, Snake, Sudoku, 2048, Tetris, Mines, etc.", accent=BRIGHT_GOLD)

arrow(6.05, 5.95, 4.5, 5.2)
arrow(9.95, 5.95, 11.5, 5.2)
arrow(7.5, 4.5, 8.5, 4.5)

# Execution Context & Preemptive Tasks
box(4.5, 2.05, 7.0, 1.1, "INTERRUPT CONTEXT SWITCHING",
    "Stack Frame Preservation • Rust timer_schedule • iretq Dispatch")
arrow(4.5, 3.85, 6.5, 3.15)
arrow(11.5, 3.85, 9.5, 3.15)

box(1.0, 0.45, 6.5, 1.1, "TASK 1: S-SHELL CLI & SYSTEM SERVICES",
    "Priority 5 • Interactive Key Buffer • S-Pkg Package Hub", accent=GREEN)
box(8.5, 0.45, 6.5, 1.1, "TASK 2: ROYAL DESKTOP COMPOSITOR",
    "Priority 5 • Backbuffer Blit • Mouse & Window Management", accent=GREEN)

arrow(6.0, 2.05, 4.25, 1.55)
arrow(10.0, 2.05, 11.75, 1.55)

ax.text(8, 1.72,
        "CA-6 TEST ENGINE: Formal Test Cases TC1 to TC6 Verified • 100% Deterministic Execution",
        ha="center", fontsize=9.5, color=BRIGHT_GOLD, fontweight="bold")

fig.tight_layout()
fig.savefig(OUT / "micro_os_architecture.png", dpi=200, facecolor=DARK_BG)
fig.savefig(OUT / "micro_os_architecture.svg", facecolor=DARK_BG)
plt.close(fig)

print("Generated:")
print(OUT / "micro_os_architecture.png")
print(OUT / "micro_os_architecture.svg")
