#!/usr/bin/env python3
"""SRIKESH OS Hybrid — Lightweight System & Hardware Monitor (GTK3)."""
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib
import os
import subprocess


CSS = b"""
window { background-color: #141923; color: #F1F5F9; font-family: "DejaVu Sans", sans-serif; }
.hdr { font-size: 16px; font-weight: 700; color: #38BDF8; }
.card { background-color: #1E242E; border: 1px solid #334155; border-radius: 8px; padding: 12px; }
.mono { font-family: "DejaVu Sans Mono", monospace; font-size: 11px; color: #CBD5E1; }
"""


def run_out(cmd):
    try:
        return subprocess.check_output(cmd, text=True, stderr=subprocess.STDOUT, timeout=4).strip()
    except Exception as e:
        return f"(unavailable: {e})"


class SysMon(Gtk.Window):
    def __init__(self):
        super().__init__(title="SRIKESH OS — System & Hardware Monitor")
        self.set_default_size(640, 460)
        self.set_position(Gtk.WindowPosition.CENTER)

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.set_margin_top(14)
        box.set_margin_bottom(14)
        box.set_margin_start(14)
        box.set_margin_end(14)
        self.add(box)

        hdr = Gtk.Label(label="SRIKESH OS Hybrid  •  Live Hardware & Network Monitor")
        hdr.set_xalign(0.0)
        hdr.get_style_context().add_class("hdr")
        box.pack_start(hdr, False, False, 0)

        scroll = Gtk.ScrolledWindow()
        scroll.set_vexpand(True)
        self.info_lbl = Gtk.Label()
        self.info_lbl.set_xalign(0.0)
        self.info_lbl.set_yalign(0.0)
        self.info_lbl.set_selectable(True)
        self.info_lbl.get_style_context().add_class("mono")
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        card.get_style_context().add_class("card")
        card.pack_start(self.info_lbl, True, True, 0)
        scroll.add(card)
        box.pack_start(scroll, True, True, 0)

        self.refresh()
        GLib.timeout_add_seconds(3, self.refresh)

    def refresh(self):
        uname = run_out(["uname", "-sr"])
        mem = run_out(["free", "-h"])
        nm = run_out(["nmcli", "dev", "status"])
        ip = run_out(["ip", "-br", "addr"])
        pci = run_out(["bash", "-c", "lspci | grep -iE 'network|wireless|ethernet|realtek|vga|audio' || true"])
        fw = "PRESENT" if os.path.exists("/lib/firmware/rtw88/rtw8821c_fw.bin") else "MISSING"
        mod = run_out(["bash", "-c", "modinfo -F filename rtw88_8821ce 2>/dev/null || echo 'not found'"])

        text = (
            f"Kernel         : {uname}\n"
            f"RTL8821CE Drv  : {mod}\n"
            f"RTL8821CE FW   : /lib/firmware/rtw88/rtw8821c_fw.bin [{fw}]\n\n"
            f"=== MEMORY ===\n{mem}\n\n"
            f"=== NETWORK INTERFACES (NetworkManager) ===\n{nm}\n\n"
            f"=== IP ADDRESSES ===\n{ip}\n\n"
            f"=== PCI HARDWARE (Wi-Fi / Ethernet / GPU / Audio) ===\n{pci}\n"
        )
        self.info_lbl.set_text(text)
        return True


def main():
    p = Gtk.CssProvider()
    p.load_from_data(CSS)
    Gtk.StyleContext.add_provider_for_screen(
        Gtk.Window().get_screen(), p, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )
    w = SysMon()
    w.connect("destroy", Gtk.main_quit)
    w.show_all()
    Gtk.main()


if __name__ == "__main__":
    main()
