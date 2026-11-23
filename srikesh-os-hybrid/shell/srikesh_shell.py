#!/usr/bin/env python3
"""
SRIKESH OS Hybrid — Custom Modern Desktop Shell (GTK3)
- Windows-11-inspired Bottom Taskbar & System Tray
- Modern Start Menu / App Launcher
- Desktop Shortcuts
- Exact Windows-11-Style Real Wi-Fi Quick Panel backed by NetworkManager (nmcli)
"""

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib, Pango

import glob
import os
import signal
import subprocess
import threading
import time
from datetime import datetime


CSS = b"""
window, .panel-bg {
    background-color: #141923;
    color: #F1F5F9;
    font-family: "DejaVu Sans", "Liberation Sans", sans-serif;
}
button {
    background-image: none;
    box-shadow: none;
    text-shadow: none;
}
.taskbar {
    background-color: #111827;
    border-top: 1px solid #2A3447;
}
.tb-btn {
    background-color: #1E293B;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 4px 12px;
    color: #F1F5F9;
    font-size: 13px;
    font-weight: 600;
}
.tb-btn:hover {
    background-color: #0EA5E9;
    border-color: #38BDF8;
    color: #FFFFFF;
}
.tb-start {
    background-color: #0284C7;
    border: 1px solid #38BDF8;
    border-radius: 6px;
    padding: 4px 14px;
    color: #FFFFFF;
    font-size: 13px;
    font-weight: 700;
}
.tb-start:hover {
    background-color: #0EA5E9;
    color: #FFFFFF;
}
.tray-btn {
    background-color: #1E293B;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 4px 10px;
    color: #E2E8F0;
    font-size: 12px;
    font-weight: 600;
}
.tray-btn:hover {
    background-color: #334155;
    color: #FFFFFF;
}
.flyout {
    background-color: #1E242E;
    border: 1px solid #38BDF8;
    border-radius: 10px;
}
.flyout-header {
    padding: 12px 14px 8px 14px;
    border-bottom: 1px solid #2B3445;
}
.flyout-title {
    font-size: 15px;
    font-weight: 700;
    color: #F8FAFC;
}
.flyout-footer {
    background-color: #171C24;
    border-top: 1px solid #2B3445;
    padding: 10px 14px;
}
.wifi-card {
    background-color: #1E242E;
    border-radius: 6px;
    padding: 10px 12px;
    margin: 3px 8px;
}
.wifi-card:hover {
    background-color: #273040;
}
.wifi-card-selected {
    background-color: #293241;
    border-left: 3px solid #38BDF8;
    border-radius: 6px;
    padding: 12px;
    margin: 4px 8px;
}
.wifi-ssid {
    font-size: 13px;
    font-weight: 700;
    color: #F8FAFC;
}
.wifi-sub {
    font-size: 11px;
    color: #94A3B8;
}
.wifi-err {
    font-size: 11px;
    color: #F87171;
    font-weight: 600;
}
.wifi-ok {
    font-size: 11px;
    color: #38BDF8;
    font-weight: 600;
}
.pw-entry {
    background-color: #161B24;
    color: #F8FAFC;
    border: 1px solid #334155;
    border-bottom: 2px solid #38BDF8;
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 13px;
}
.btn-primary {
    background-color: #0EA5E9;
    color: #FFFFFF;
    border: none;
    border-radius: 5px;
    padding: 6px 16px;
    font-size: 12px;
    font-weight: 700;
}
.btn-primary:hover {
    background-color: #38BDF8;
}
.btn-secondary {
    background-color: #334155;
    color: #F1F5F9;
    border: none;
    border-radius: 5px;
    padding: 6px 14px;
    font-size: 12px;
    font-weight: 600;
}
.btn-secondary:hover {
    background-color: #475569;
}
.btn-danger {
    background-color: #991B1B;
    color: #FFFFFF;
    border: none;
    border-radius: 5px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
}
.btn-danger:hover {
    background-color: #DC2626;
}
.desktop-icon-win {
    background-color: #111827;
    border: 1px solid #2A3447;
    border-radius: 10px;
}
.desk-icon {
    background-color: #1E293B;
    border: 1px solid #334155;
    border-radius: 8px;
    color: #F8FAFC;
    padding: 8px 6px;
    font-size: 12px;
    font-weight: 600;
}
.desk-icon:hover {
    background-color: #0284C7;
    border-color: #38BDF8;
    color: #FFFFFF;
}
.start-app {
    background-color: #242C3B;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 12px 8px;
    color: #F8FAFC;
    font-size: 12px;
    font-weight: 600;
}
.start-app:hover {
    background-color: #0284C7;
    border-color: #38BDF8;
    color: #FFFFFF;
}
.welcome-bg {
    background-color: #090E1A;
}
.welcome-tile-a { background-color: #0EA5E9; border-radius: 8px; }
.welcome-tile-b { background-color: #38BDF8; border-radius: 8px; }
.welcome-tile-c { background-color: #6366F1; border-radius: 8px; }
.welcome-tile-d { background-color: #0284C7; border-radius: 8px; }
.welcome-title {
    font-size: 46px;
    font-weight: 700;
    color: #F8FAFC;
}
.welcome-tagline {
    font-size: 20px;
    font-weight: 400;
    color: #38BDF8;
}
.welcome-status {
    font-size: 13px;
    color: #94A3B8;
}
progressbar trough {
    background-color: #1E293B;
    min-height: 4px;
    border-radius: 2px;
}
progressbar progress {
    background-color: #38BDF8;
    min-height: 4px;
    border-radius: 2px;
}
switch {
    background-color: #475569;
    border-radius: 12px;
}
switch:checked {
    background-color: #0EA5E9;
}
"""


def run_cmd(args, timeout=15):
    try:
        r = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
        return r.returncode, r.stdout.strip(), r.stderr.strip()
    except Exception as e:
        return 1, "", str(e)


def launch_browser(url="https://www.google.com"):
    cmd = [
        "chromium",
        "--no-first-run",
        "--disable-infobars",
        "--disable-default-apps",
        "--password-store=basic",
        url,
    ]
    subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def launch_app(cmd_list):
    try:
        subprocess.Popen(cmd_list, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass


def get_wifi_device():
    rc, out, _ = run_cmd(["nmcli", "-t", "-f", "DEVICE,TYPE,STATE", "dev", "status"], timeout=5)
    if rc != 0:
        return None, None
    for line in out.splitlines():
        parts = line.split(":")
        if len(parts) >= 3 and parts[1] == "wifi":
            return parts[0], parts[2]
    return None, None


def get_ethernet_status():
    rc, out, _ = run_cmd(["nmcli", "-t", "-f", "DEVICE,TYPE,STATE", "dev", "status"], timeout=5)
    if rc != 0:
        return None, None
    for line in out.splitlines():
        parts = line.split(":")
        if len(parts) >= 3 and parts[1] == "ethernet" and parts[2] == "connected":
            dev = parts[0]
            ip = get_device_ip(dev)
            return dev, ip
    return None, None


def get_device_ip(dev):
    if not dev:
        return ""
    rc, out, _ = run_cmd(["nmcli", "-t", "-f", "IP4.ADDRESS", "dev", "show", dev], timeout=5)
    if rc == 0:
        for line in out.splitlines():
            if line.startswith("IP4.ADDRESS"):
                val = line.split(":", 1)[1].strip()
                return val.split("/")[0]
    return ""


def get_saved_connections():
    rc, out, _ = run_cmd(["nmcli", "-t", "-f", "NAME,TYPE", "con", "show"], timeout=5)
    saved = set()
    if rc == 0:
        for line in out.splitlines():
            parts = line.rsplit(":", 1)
            if len(parts) == 2 and parts[1] == "802-11-wireless":
                saved.add(parts[0])
    return saved


def is_wifi_enabled():
    rc, out, _ = run_cmd(["nmcli", "radio", "wifi"], timeout=5)
    return rc == 0 and out.strip().lower() == "enabled"


def scan_wifi_networks(rescan=True):
    """Returns list of dicts: {in_use, ssid, signal, security, saved}."""
    args = ["nmcli", "-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "dev", "wifi", "list"]
    if rescan:
        args += ["--rescan", "yes"]
    rc, out, _ = run_cmd(args, timeout=20)
    saved = get_saved_connections()
    if rc != 0:
        return []
    seen = {}
    for line in out.splitlines():
        if not line:
            continue
        # Escaped colons in SSID: nmcli uses \:
        line_safe = line.replace("\\:", "\x00")
        parts = line_safe.split(":")
        if len(parts) < 4:
            continue
        in_use = parts[0].strip() == "*"
        ssid = parts[1].replace("\x00", ":").strip()
        if not ssid:
            continue
        try:
            sig = int(parts[2].strip())
        except ValueError:
            sig = 0
        sec = parts[3].strip()
        if ssid not in seen or in_use or sig > seen[ssid]["signal"]:
            seen[ssid] = {
                "in_use": in_use,
                "ssid": ssid,
                "signal": sig,
                "security": sec if sec and sec != "--" else "Open",
                "saved": ssid in saved,
            }
    nets = list(seen.values())
    nets.sort(key=lambda n: (not n["in_use"], -n["signal"], n["ssid"].lower()))
    return nets


def signal_icon(sig, secured):
    bars = "▂▄▆█" if sig >= 75 else ("▂▄▆_" if sig >= 50 else ("▂▄__" if sig >= 25 else "▂___"))
    lock = " [WPA]" if secured else ""
    return f"{bars}{lock}"


class WifiPanel(Gtk.Window):
    def __init__(self, parent_shell):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.shell = parent_shell
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_default_size(360, 470)
        self.set_type_hint(Gdk.WindowTypeHint.DIALOG)
        self.get_style_context().add_class("flyout")

        self.selected_ssid = None
        self.connecting_ssid = None
        self.error_ssid = None
        self.error_msg = ""
        self.networks = []
        self.scanning = False
        self.connect_proc = None
        self.pulse_timer = None
        self.active_progress = None

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(root)

        # Header: <- Wi-Fi    [Toggle]
        header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        header.get_style_context().add_class("flyout-header")
        back_btn = Gtk.Button(label="←")
        back_btn.get_style_context().add_class("tray-btn")
        back_btn.connect("clicked", lambda *_: self.hide())
        header.pack_start(back_btn, False, False, 0)

        title = Gtk.Label(label="Wi-Fi")
        title.get_style_context().add_class("flyout-title")
        header.pack_start(title, False, False, 4)

        self.wifi_switch = Gtk.Switch()
        self.wifi_switch.set_active(is_wifi_enabled())
        self.wifi_switch.connect("notify::active", self.on_wifi_toggle)
        header.pack_end(self.wifi_switch, False, False, 0)
        root.pack_start(header, False, False, 0)

        # Status banner (scanning / ethernet fallback)
        self.status_label = Gtk.Label(label="Scanning for Wi-Fi networks...")
        self.status_label.set_xalign(0.0)
        self.status_label.get_style_context().add_class("wifi-sub")
        self.status_label.set_margin_start(14)
        self.status_label.set_margin_end(14)
        self.status_label.set_margin_top(6)
        self.status_label.set_margin_bottom(4)
        root.pack_start(self.status_label, False, False, 0)

        # Scrollable network list
        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroll.set_vexpand(True)
        self.list_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        scroll.add(self.list_box)
        root.pack_start(scroll, True, True, 0)

        # Footer: More Wi-Fi settings    [Refresh]
        footer = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        footer.get_style_context().add_class("flyout-footer")

        more_btn = Gtk.Button(label="More Wi-Fi settings")
        more_btn.get_style_context().add_class("tray-btn")
        more_btn.connect("clicked", lambda *_: launch_app(["python3", "/opt/srikesh/srikesh_sysmon.py"]))
        footer.pack_start(more_btn, False, False, 0)

        refresh_btn = Gtk.Button(label="↻ Refresh")
        refresh_btn.get_style_context().add_class("tray-btn")
        refresh_btn.connect("clicked", lambda *_: self.refresh_async(rescan=True))
        footer.pack_end(refresh_btn, False, False, 0)
        root.pack_end(footer, False, False, 0)

        self.connect("key-press-event", self.on_key_press)

    def on_key_press(self, widget, event):
        if event.keyval == Gdk.KEY_Escape:
            self.hide()
            return True
        return False

    def position_bottom_right(self):
        screen = Gdk.Screen.get_default()
        sw = screen.get_width()
        sh = screen.get_height()
        self.move(sw - 376, sh - 532)

    def open_panel(self):
        self.position_bottom_right()
        self.show_all()
        self.present()
        self.refresh_async(rescan=True)

    def on_wifi_toggle(self, switch, _pspec):
        state = "on" if switch.get_active() else "off"
        def worker():
            run_cmd(["nmcli", "radio", "wifi", state], timeout=8)
            time.sleep(0.8)
            GLib.idle_add(self.refresh_async, True)
        threading.Thread(target=worker, daemon=True).start()

    def refresh_async(self, rescan=True):
        if self.scanning:
            return
        self.scanning = True
        self.status_label.set_text("Scanning nearby Wi-Fi networks...")

        def worker():
            dev, dev_state = get_wifi_device()
            eth_dev, eth_ip = get_ethernet_status()
            enabled = is_wifi_enabled()
            nets = scan_wifi_networks(rescan=rescan) if (dev and enabled) else []
            dev_ip = get_device_ip(dev) if dev else ""
            GLib.idle_add(self._on_scan_done, dev, dev_state, dev_ip, eth_dev, eth_ip, enabled, nets)

        threading.Thread(target=worker, daemon=True).start()

    def _on_scan_done(self, dev, dev_state, dev_ip, eth_dev, eth_ip, enabled, nets):
        self.scanning = False
        self.wifi_dev = dev
        self.wifi_ip = dev_ip
        self.eth_dev = eth_dev
        self.eth_ip = eth_ip
        self.wifi_enabled = enabled
        self.networks = nets

        self.wifi_switch.handler_block_by_func(self.on_wifi_toggle)
        self.wifi_switch.set_active(enabled)
        self.wifi_switch.handler_unblock_by_func(self.on_wifi_toggle)

        if not dev:
            if eth_dev:
                self.status_label.set_text(f"Ethernet Connected ({eth_dev}: {eth_ip}) • No Wi-Fi PCI card in VM")
            else:
                self.status_label.set_text("No Wi-Fi adapter detected on this system")
        elif not enabled:
            self.status_label.set_text("Wi-Fi is turned off")
        else:
            self.status_label.set_text(f"Adapter: {dev} • {len(nets)} network(s) found")

        self.rebuild_list()
        self.shell.update_tray_network()

    def rebuild_list(self):
        for child in self.list_box.get_children():
            self.list_box.remove(child)
        self.active_progress = None

        if not getattr(self, "wifi_dev", None):
            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            card.get_style_context().add_class("wifi-card-selected")
            t1 = Gtk.Label(label="No Wi-Fi Hardware Detected")
            t1.set_xalign(0.0)
            t1.get_style_context().add_class("wifi-ssid")
            card.pack_start(t1, False, False, 0)

            msg = (
                f"Active Ethernet: {self.eth_dev} ({self.eth_ip})\n"
                "Internet is live via Ethernet in this Virtual Machine.\n\n"
                "On your Dell Vostro 3520 laptop, the Realtek RTL8821CE\n"
                "driver (rtw88_8821ce) & firmware (rtw8821c_fw.bin) load\n"
                "automatically and list real nearby Wi-Fi networks here."
                if getattr(self, "eth_dev", None)
                else "No PCIe/USB Wi-Fi adapter found.\nInsert a Wi-Fi adapter or boot on Dell Vostro 3520."
            )
            t2 = Gtk.Label(label=msg)
            t2.set_xalign(0.0)
            t2.set_line_wrap(True)
            t2.get_style_context().add_class("wifi-sub")
            card.pack_start(t2, False, False, 0)
            self.list_box.pack_start(card, False, False, 4)
            self.list_box.show_all()
            return

        if not getattr(self, "wifi_enabled", True):
            lbl = Gtk.Label(label="Turn Wi-Fi ON above to scan nearby networks.")
            lbl.get_style_context().add_class("wifi-sub")
            lbl.set_margin_top(24)
            self.list_box.pack_start(lbl, False, False, 8)
            self.list_box.show_all()
            return

        if not self.networks:
            lbl = Gtk.Label(label="No nearby Wi-Fi networks found. Click Refresh to rescan.")
            lbl.get_style_context().add_class("wifi-sub")
            lbl.set_margin_top(24)
            self.list_box.pack_start(lbl, False, False, 8)
            self.list_box.show_all()
            return

        for net in self.networks:
            ssid = net["ssid"]
            secured = net["security"] != "Open"
            is_connected = net["in_use"]
            is_connecting = (self.connecting_ssid == ssid)
            is_selected = (self.selected_ssid == ssid) or is_connected or is_connecting

            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            card.get_style_context().add_class("wifi-card-selected" if is_selected else "wifi-card")

            # Clickable header row
            ev = Gtk.EventBox()
            ev.set_visible_window(False)
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)

            ico = Gtk.Label(label=signal_icon(net["signal"], secured))
            ico.get_style_context().add_class("wifi-ok" if is_connected else "wifi-sub")
            row.pack_start(ico, False, False, 0)

            info_col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            name_lbl = Gtk.Label(label=ssid)
            name_lbl.set_xalign(0.0)
            name_lbl.set_ellipsize(Pango.EllipsizeMode.END)
            name_lbl.get_style_context().add_class("wifi-ssid")
            info_col.pack_start(name_lbl, False, False, 0)

            if is_connected:
                sub_txt = f"Connected, {'secured' if secured else 'open'}"
                if self.wifi_ip:
                    sub_txt += f" • {self.wifi_ip}"
                sub_lbl = Gtk.Label(label=sub_txt)
                sub_lbl.set_xalign(0.0)
                sub_lbl.get_style_context().add_class("wifi-ok")
                info_col.pack_start(sub_lbl, False, False, 0)
            elif is_connecting or is_selected:
                sub_lbl = Gtk.Label(label="Secured" if secured else "Open")
                sub_lbl.set_xalign(0.0)
                sub_lbl.get_style_context().add_class("wifi-sub")
                info_col.pack_start(sub_lbl, False, False, 0)

            row.pack_start(info_col, True, True, 0)

            sig_lbl = Gtk.Label(label=f"{net['signal']}%")
            sig_lbl.get_style_context().add_class("wifi-sub")
            row.pack_end(sig_lbl, False, False, 0)

            ev.add(row)
            ev.connect("button-press-event", self.on_select_network, ssid)
            card.pack_start(ev, False, False, 0)

            # Expanded body ONLY for connected / connecting / explicitly selected network
            if is_connected:
                btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
                disc_btn = Gtk.Button(label="Disconnect")
                disc_btn.get_style_context().add_class("btn-secondary")
                disc_btn.connect("clicked", self.on_disconnect, ssid)
                btn_row.pack_end(disc_btn, False, False, 0)
                card.pack_start(btn_row, False, False, 4)

            elif is_connecting:
                prog = Gtk.ProgressBar()
                prog.pulse()
                self.active_progress = prog
                card.pack_start(prog, False, False, 2)

                st = Gtk.Label(label="Checking network requirements...")
                st.set_xalign(0.0)
                st.get_style_context().add_class("wifi-sub")
                card.pack_start(st, False, False, 0)

                btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
                cancel_btn = Gtk.Button(label="Cancel")
                cancel_btn.get_style_context().add_class("btn-secondary")
                cancel_btn.connect("clicked", self.on_cancel_connect)
                btn_row.pack_end(cancel_btn, False, False, 0)
                card.pack_start(btn_row, False, False, 2)

            elif self.selected_ssid == ssid:
                if self.error_ssid == ssid and self.error_msg:
                    err_lbl = Gtk.Label(label=self.error_msg)
                    err_lbl.set_xalign(0.0)
                    err_lbl.set_line_wrap(True)
                    err_lbl.get_style_context().add_class("wifi-err")
                    card.pack_start(err_lbl, False, False, 2)

                if net["saved"] and self.error_ssid != ssid:
                    saved_lbl = Gtk.Label(label="Saved network — click Connect to rejoin.")
                    saved_lbl.set_xalign(0.0)
                    saved_lbl.get_style_context().add_class("wifi-sub")
                    card.pack_start(saved_lbl, False, False, 2)

                    btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
                    conn_btn = Gtk.Button(label="Connect")
                    conn_btn.get_style_context().add_class("btn-primary")
                    conn_btn.connect("clicked", lambda *_, s=ssid: self.start_connect(s, None, saved=True))
                    forget_btn = Gtk.Button(label="Forget")
                    forget_btn.get_style_context().add_class("btn-secondary")
                    forget_btn.connect("clicked", self.on_forget_network, ssid)
                    btn_row.pack_start(conn_btn, True, True, 0)
                    btn_row.pack_start(forget_btn, False, False, 0)
                    card.pack_start(btn_row, False, False, 4)

                elif secured:
                    pw_lbl = Gtk.Label(label="Enter the password")
                    pw_lbl.set_xalign(0.0)
                    pw_lbl.get_style_context().add_class("wifi-sub")
                    card.pack_start(pw_lbl, False, False, 2)

                    entry = Gtk.Entry()
                    entry.set_visibility(False)
                    entry.set_invisible_char("•")
                    entry.set_placeholder_text("Network security key")
                    entry.get_style_context().add_class("pw-entry")
                    entry.connect("activate", lambda e, s=ssid: self.start_connect(s, e.get_text(), saved=False))
                    card.pack_start(entry, False, False, 2)
                    GLib.idle_add(entry.grab_focus)

                    btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
                    next_btn = Gtk.Button(label="Next")
                    next_btn.get_style_context().add_class("btn-primary")
                    next_btn.connect("clicked", lambda *_, s=ssid, e=entry: self.start_connect(s, e.get_text(), saved=False))
                    cancel_btn = Gtk.Button(label="Cancel")
                    cancel_btn.get_style_context().add_class("btn-secondary")
                    cancel_btn.connect("clicked", self.on_clear_selection)
                    btn_row.pack_start(next_btn, True, True, 0)
                    btn_row.pack_start(cancel_btn, True, True, 0)
                    card.pack_start(btn_row, False, False, 4)

                else:
                    btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
                    conn_btn = Gtk.Button(label="Connect")
                    conn_btn.get_style_context().add_class("btn-primary")
                    conn_btn.connect("clicked", lambda *_, s=ssid: self.start_connect(s, "", saved=False))
                    cancel_btn = Gtk.Button(label="Cancel")
                    cancel_btn.get_style_context().add_class("btn-secondary")
                    cancel_btn.connect("clicked", self.on_clear_selection)
                    btn_row.pack_start(conn_btn, True, True, 0)
                    btn_row.pack_start(cancel_btn, True, True, 0)
                    card.pack_start(btn_row, False, False, 4)

            self.list_box.pack_start(card, False, False, 2)

        self.list_box.show_all()

    def on_select_network(self, _widget, _event, ssid):
        if self.connecting_ssid:
            return True
        if self.selected_ssid == ssid:
            self.selected_ssid = None
        else:
            self.selected_ssid = ssid
        self.rebuild_list()
        return True

    def on_clear_selection(self, *_):
        self.selected_ssid = None
        self.error_ssid = None
        self.error_msg = ""
        self.rebuild_list()

    def on_disconnect(self, _btn, ssid):
        def worker():
            run_cmd(["nmcli", "con", "down", ssid], timeout=10)
            if getattr(self, "wifi_dev", None):
                run_cmd(["nmcli", "dev", "disconnect", self.wifi_dev], timeout=10)
            GLib.idle_add(self.refresh_async, False)
        threading.Thread(target=worker, daemon=True).start()

    def on_forget_network(self, _btn, ssid):
        def worker():
            run_cmd(["nmcli", "con", "delete", ssid], timeout=10)
            GLib.idle_add(self.refresh_async, False)
        threading.Thread(target=worker, daemon=True).start()

    def on_cancel_connect(self, *_):
        if self.connect_proc and self.connect_proc.poll() is None:
            try:
                self.connect_proc.terminate()
            except Exception:
                pass
        self.connecting_ssid = None
        self.rebuild_list()

    def _pulse_tick(self):
        if self.active_progress and self.connecting_ssid:
            self.active_progress.pulse()
            return True
        return False

    def start_connect(self, ssid, password, saved=False):
        if not saved and password is not None and len(password) > 0 and len(password) < 8:
            self.error_ssid = ssid
            self.error_msg = "Password must be at least 8 characters for WPA2/WPA3."
            self.rebuild_list()
            return

        self.connecting_ssid = ssid
        self.error_ssid = None
        self.error_msg = ""
        self.rebuild_list()
        GLib.timeout_add(120, self._pulse_tick)

        def worker():
            if saved:
                cmd = ["nmcli", "con", "up", ssid]
            else:
                # Delete stale profile with same SSID first so new password is used
                run_cmd(["nmcli", "con", "delete", ssid], timeout=5)
                cmd = ["nmcli", "dev", "wifi", "connect", ssid]
                if password:
                    cmd += ["password", password]
            try:
                self.connect_proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                out, err = self.connect_proc.communicate(timeout=35)
                rc = self.connect_proc.returncode
            except subprocess.TimeoutExpired:
                if self.connect_proc:
                    self.connect_proc.kill()
                rc, out, err = 1, "", "Connection timed out."
            except Exception as e:
                rc, out, err = 1, "", str(e)

            GLib.idle_add(self._on_connect_done, ssid, rc, (err or out).strip())

        threading.Thread(target=worker, daemon=True).start()

    def _on_connect_done(self, ssid, rc, detail):
        self.connecting_ssid = None
        self.connect_proc = None
        if rc == 0:
            self.selected_ssid = None
            self.error_ssid = None
            self.error_msg = ""
            self.refresh_async(rescan=False)
        else:
            self.selected_ssid = ssid
            self.error_ssid = ssid
            if "Secrets were required" in detail or "802-11-wireless-security" in detail:
                self.error_msg = "Authentication failed — incorrect Wi-Fi password."
            else:
                self.error_msg = f"Connection failed: {detail[:90]}"
            self.rebuild_list()


class StartMenu(Gtk.Window):
    def __init__(self, shell):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.shell = shell
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_default_size(420, 455)
        self.set_type_hint(Gdk.WindowTypeHint.DIALOG)
        self.get_style_context().add_class("flyout")

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        root.set_margin_top(14)
        root.set_margin_bottom(0)
        root.set_margin_start(14)
        root.set_margin_end(14)
        self.add(root)

        hdr = Gtk.Label(label="SRIKESH OS  •  Applications")
        hdr.set_xalign(0.0)
        hdr.get_style_context().add_class("flyout-title")
        root.pack_start(hdr, False, False, 0)

        self.search = Gtk.Entry()
        self.search.set_placeholder_text("Search applications...")
        self.search.get_style_context().add_class("pw-entry")
        self.search.connect("changed", self.on_filter)
        root.pack_start(self.search, False, False, 2)

        self.apps = [
            ("◉  Browser", lambda: launch_browser("https://www.google.com")),
            ("◎  Google", lambda: launch_browser("https://www.google.com")),
            ("▶  YouTube", lambda: launch_browser("https://www.youtube.com")),
            ("❖  Snap Store", lambda: launch_app(["python3", "/opt/srikesh/srikesh_store.py"])),
            ("▤  Files", lambda: launch_app(["pcmanfm"])),
            (">_  Terminal", lambda: launch_app(["lxterminal"])),
            ("▂▄▆  Wi-Fi Settings", lambda: self.shell.toggle_wifi_panel()),
            ("◈  System Monitor", lambda: launch_app(["python3", "/opt/srikesh/srikesh_sysmon.py"])),
            ("★  Chrome Dino", lambda: launch_browser("chrome://dino")),
        ]

        self.grid = Gtk.Grid()
        self.grid.set_row_spacing(8)
        self.grid.set_column_spacing(8)
        self.grid.set_column_homogeneous(True)
        root.pack_start(self.grid, True, True, 4)
        self.populate_grid("")

        footer = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        footer.get_style_context().add_class("flyout-footer")
        footer.set_margin_start(-14)
        footer.set_margin_end(-14)

        user_lbl = Gtk.Label(label="SRIKESH OS Hybrid")
        user_lbl.get_style_context().add_class("wifi-sub")
        footer.pack_start(user_lbl, False, False, 4)

        pwr_btn = Gtk.Button(label="⏻ Power Off")
        pwr_btn.get_style_context().add_class("btn-danger")
        pwr_btn.connect("clicked", lambda *_: launch_app(["systemctl", "poweroff"]))
        footer.pack_end(pwr_btn, False, False, 0)

        reb_btn = Gtk.Button(label="↻ Reboot")
        reb_btn.get_style_context().add_class("btn-secondary")
        reb_btn.connect("clicked", lambda *_: launch_app(["systemctl", "reboot"]))
        footer.pack_end(reb_btn, False, False, 0)

        root.pack_end(footer, False, False, 0)
        self.connect("key-press-event", self.on_key_press)

    def populate_grid(self, query):
        for c in self.grid.get_children():
            self.grid.remove(c)
        q = query.strip().lower()
        filtered = [(name, fn) for (name, fn) in self.apps if q in name.lower()]
        for idx, (name, fn) in enumerate(filtered):
            btn = Gtk.Button(label=name)
            btn.get_style_context().add_class("start-app")
            btn.connect("clicked", lambda *_, f=fn: (self.hide(), f()))
            self.grid.attach(btn, idx % 2, idx // 2, 1, 1)
        self.grid.show_all()

    def on_filter(self, entry):
        self.populate_grid(entry.get_text())

    def on_key_press(self, _w, ev):
        if ev.keyval == Gdk.KEY_Escape:
            self.hide()
            return True
        return False

    def open_menu(self):
        screen = Gdk.Screen.get_default()
        sh = screen.get_height()
        self.move(12, sh - 510)
        self.search.set_text("")
        self.show_all()
        self.present()
        self.search.grab_focus()


class DesktopIcons(Gtk.Window):
    def __init__(self, shell):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.shell = shell
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_below(True)
        self.set_type_hint(Gdk.WindowTypeHint.DESKTOP)
        self.set_app_paintable(True)
        visual = self.get_screen().get_rgba_visual()
        if visual:
            self.set_visual(visual)
        self.get_style_context().add_class("desktop-icon-win")
        self.move(22, 24)

        col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        col.set_margin_top(10)
        col.set_margin_bottom(10)
        col.set_margin_start(10)
        col.set_margin_end(10)
        items = [
            ("◉\nBrowser", lambda: launch_browser("https://www.google.com")),
            ("▶\nYouTube", lambda: launch_browser("https://www.youtube.com")),
            ("❖\nSnap Store", lambda: launch_app(["python3", "/opt/srikesh/srikesh_store.py"])),
            ("▤\nFiles", lambda: launch_app(["pcmanfm"])),
            (">_\nTerminal", lambda: launch_app(["lxterminal"])),
            ("▂▄▆\nWi-Fi", lambda: self.shell.toggle_wifi_panel()),
            ("◈\nMonitor", lambda: launch_app(["python3", "/opt/srikesh/srikesh_sysmon.py"])),
        ]
        for label, fn in items:
            b = Gtk.Button(label=label)
            b.set_size_request(86, 62)
            b.get_style_context().add_class("desk-icon")
            b.connect("clicked", lambda *_, f=fn: f())
            col.pack_start(b, False, False, 0)
        self.add(col)


class SrikeshTaskbar(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_type_hint(Gdk.WindowTypeHint.DOCK)
        self.get_style_context().add_class("taskbar")

        screen = Gdk.Screen.get_default()
        sw = screen.get_width()
        sh = screen.get_height()
        self.bar_h = 48
        self.set_default_size(sw, self.bar_h)
        self.move(0, sh - self.bar_h)

        self.wifi_panel = WifiPanel(self)
        self.start_menu = StartMenu(self)
        self.desktop_icons = DesktopIcons(self)

        bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        bar.set_margin_start(10)
        bar.set_margin_end(10)
        bar.set_margin_top(4)
        bar.set_margin_bottom(4)
        self.add(bar)

        # Left / Center Pinned Dock
        start_btn = Gtk.Button(label="⊞  SRIKESH")
        start_btn.get_style_context().add_class("tb-start")
        start_btn.connect("clicked", lambda *_: self.toggle_start_menu())
        bar.pack_start(start_btn, False, False, 2)

        pinned = [
            ("◉ Browser", lambda: launch_browser("https://www.google.com")),
            ("▶ YouTube", lambda: launch_browser("https://www.youtube.com")),
            ("❖ Snap Store", lambda: launch_app(["python3", "/opt/srikesh/srikesh_store.py"])),
            ("▤ Files", lambda: launch_app(["pcmanfm"])),
            (">_ Terminal", lambda: launch_app(["lxterminal"])),
            ("◈ Monitor", lambda: launch_app(["python3", "/opt/srikesh/srikesh_sysmon.py"])),
        ]
        for label, fn in pinned:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("tb-btn")
            b.connect("clicked", lambda *_, f=fn: f())
            bar.pack_start(b, False, False, 2)

        # Right System Tray
        self.clock_lbl = Gtk.Label()
        self.clock_lbl.get_style_context().add_class("tray-btn")
        bar.pack_end(self.clock_lbl, False, False, 4)

        self.bat_lbl = Gtk.Label(label="⚡ AC")
        self.bat_lbl.get_style_context().add_class("tray-btn")
        bar.pack_end(self.bat_lbl, False, False, 2)

        self.net_btn = Gtk.Button(label="▂▄▆ Wi-Fi")
        self.net_btn.get_style_context().add_class("tray-btn")
        self.net_btn.connect("clicked", lambda *_: self.toggle_wifi_panel())
        bar.pack_end(self.net_btn, False, False, 2)

        self.connect("realize", self.set_strut)
        self.update_clock()
        self.update_tray_network()
        GLib.timeout_add_seconds(1, self.update_clock)
        GLib.timeout_add_seconds(5, self.update_tray_network)

    def set_strut(self, *_):
        gdk_win = self.get_window()
        if not gdk_win:
            return
        screen = Gdk.Screen.get_default()
        sw = screen.get_width()
        Gdk.property_change(
            gdk_win,
            Gdk.Atom.intern("_NET_WM_STRUT_PARTIAL", False),
            Gdk.Atom.intern("CARDINAL", False),
            32,
            Gdk.PropMode.REPLACE,
            [0, 0, 0, self.bar_h, 0, 0, 0, 0, 0, 0, 0, sw - 1],
        )

    def toggle_wifi_panel(self):
        self.start_menu.hide()
        if self.wifi_panel.get_visible():
            self.wifi_panel.hide()
        else:
            self.wifi_panel.open_panel()

    def toggle_start_menu(self):
        self.wifi_panel.hide()
        if self.start_menu.get_visible():
            self.start_menu.hide()
        else:
            self.start_menu.open_menu()

    def update_clock(self):
        now = datetime.now()
        self.clock_lbl.set_text(now.strftime("%H:%M\n%d-%m-%Y"))
        bats = glob.glob("/sys/class/power_supply/BAT*/capacity")
        if bats:
            try:
                pct = open(bats[0]).read().strip()
                self.bat_lbl.set_text(f"⚡ {pct}%")
            except Exception:
                pass
        return True

    def update_tray_network(self):
        def worker():
            dev, state = get_wifi_device()
            eth_dev, eth_ip = get_ethernet_status()
            text = "▂▄▆ Wi-Fi"
            if dev and state == "connected":
                rc, out, _ = run_cmd(["nmcli", "-t", "-f", "ACTIVE,SSID", "dev", "wifi"], timeout=4)
                ssid = "Connected"
                for l in out.splitlines():
                    if l.startswith("yes:"):
                        ssid = l.split(":", 1)[1]
                        break
                text = f"▂▄▆ {ssid[:14]}"
            elif eth_dev:
                text = f"◉ {eth_ip or 'Ethernet'}"
            elif dev:
                text = "▂___ Wi-Fi"
            GLib.idle_add(self.net_btn.set_label, text)
        threading.Thread(target=worker, daemon=True).start()
        return True


class WelcomeOverlay(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_type_hint(Gdk.WindowTypeHint.SPLASHSCREEN)
        self.get_style_context().add_class("welcome-bg")

        screen = Gdk.Screen.get_default()
        if screen:
            self.set_default_size(screen.get_width(), screen.get_height())
        self.move(0, 0)
        self.fullscreen()

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        box.set_halign(Gtk.Align.CENTER)
        box.set_valign(Gtk.Align.CENTER)

        grid = Gtk.Grid()
        grid.set_row_spacing(8)
        grid.set_column_spacing(8)
        grid.set_halign(Gtk.Align.CENTER)
        for cls, c, r in [
            ("welcome-tile-a", 0, 0),
            ("welcome-tile-b", 1, 0),
            ("welcome-tile-c", 0, 1),
            ("welcome-tile-d", 1, 1),
        ]:
            t = Gtk.Box()
            t.set_size_request(38, 38)
            t.get_style_context().add_class(cls)
            grid.attach(t, c, r, 1, 1)
        box.pack_start(grid, False, False, 18)

        title = Gtk.Label(label="SRIKESH OS")
        title.get_style_context().add_class("welcome-title")
        box.pack_start(title, False, False, 6)

        tagline = Gtk.Label(label="Awaken the Power of Technology.")
        tagline.get_style_context().add_class("welcome-tagline")
        box.pack_start(tagline, False, False, 10)

        self.progress = Gtk.ProgressBar()
        self.progress.set_size_request(340, 8)
        self.progress.set_halign(Gtk.Align.CENTER)
        self.progress.set_fraction(0.08)
        box.pack_start(self.progress, False, False, 18)

        self.status = Gtk.Label(label="Initializing SRIKESH OS desktop...")
        self.status.get_style_context().add_class("welcome-status")
        box.pack_start(self.status, False, False, 4)

        self.add(box)
        self.connect("key-press-event", lambda *_: self.close_splash())
        self.connect("button-press-event", lambda *_: self.close_splash())

        self.step = 0
        self.total_steps = 65  # ~3.9 seconds
        self.closed = False
        GLib.timeout_add(60, self.tick)

    def close_splash(self):
        if not self.closed:
            self.closed = True
            self.destroy()

    def tick(self):
        if self.closed:
            return False
        self.step += 1
        frac = min(1.0, self.step / self.total_steps)
        self.progress.set_fraction(frac)
        if frac < 0.35:
            self.status.set_text("Initializing SRIKESH OS desktop...")
        elif frac < 0.72:
            self.status.set_text("Starting Wi-Fi & Hardware services...")
        else:
            self.status.set_text("Welcome, Srikesh")
        if self.step >= self.total_steps:
            self.close_splash()
            return False
        return True


def main():
    settings = Gtk.Settings.get_default()
    if settings:
        settings.set_property("gtk-application-prefer-dark-theme", True)
    provider = Gtk.CssProvider()
    provider.load_from_data(CSS)
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )

    tb = SrikeshTaskbar()
    tb.show_all()
    tb.desktop_icons.show_all()
    welcome = WelcomeOverlay()
    welcome.show_all()
    signal.signal(signal.SIGUSR1, lambda *_: GLib.idle_add(tb.toggle_wifi_panel))
    Gtk.main()


if __name__ == "__main__":
    main()
