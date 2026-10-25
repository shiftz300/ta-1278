#!/usr/bin/env python3
"""Enumerate cached USB device descriptors. No libusb_open, claim, or transfer."""
import ctypes as C
import datetime
import json
from pathlib import Path

class Descriptor(C.Structure):
    _fields_ = [('bLength',C.c_uint8),('bDescriptorType',C.c_uint8),('bcdUSB',C.c_uint16),
                ('bDeviceClass',C.c_uint8),('bDeviceSubClass',C.c_uint8),('bDeviceProtocol',C.c_uint8),
                ('bMaxPacketSize0',C.c_uint8),('idVendor',C.c_uint16),('idProduct',C.c_uint16),
                ('bcdDevice',C.c_uint16),('iManufacturer',C.c_uint8),('iProduct',C.c_uint8),
                ('iSerialNumber',C.c_uint8),('bNumConfigurations',C.c_uint8)]

lib = C.CDLL('/opt/homebrew/opt/libusb/lib/libusb-1.0.dylib')
lib.libusb_init.argtypes = [C.POINTER(C.c_void_p)]
lib.libusb_init.restype = C.c_int
lib.libusb_get_device_list.argtypes = [C.c_void_p, C.POINTER(C.POINTER(C.c_void_p))]
lib.libusb_get_device_list.restype = C.c_ssize_t
lib.libusb_get_device_descriptor.argtypes = [C.c_void_p, C.POINTER(Descriptor)]
lib.libusb_get_device_descriptor.restype = C.c_int
lib.libusb_free_device_list.argtypes = [C.POINTER(C.c_void_p), C.c_int]
lib.libusb_exit.argtypes = [C.c_void_p]
ctx = C.c_void_p()
status = lib.libusb_init(C.byref(ctx))
result = {'init_status': status, 'devices': [], 'method': 'cached descriptors only; no device handles or transfers'}
if status == 0:
    items = C.POINTER(C.c_void_p)()
    n = lib.libusb_get_device_list(ctx, C.byref(items))
    result['device_count_or_error'] = n
    if n >= 0:
        for i in range(n):
            d = Descriptor()
            rc = lib.libusb_get_device_descriptor(items[i], C.byref(d))
            row = {'descriptor_status': rc}
            if rc == 0:
                for field in ['idVendor','idProduct','bcdUSB','bcdDevice']:
                    row[field] = '0x%04x' % getattr(d,field)
                for field in ['bDeviceClass','bDeviceSubClass','bDeviceProtocol','bNumConfigurations']:
                    row[field] = getattr(d,field)
            result['devices'].append(row)
        lib.libusb_free_device_list(items,1)
    lib.libusb_exit(ctx)
now = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8)))
result['time_Asia_Shanghai'] = now.isoformat()
out = Path(__file__).resolve().parents[1] / 'analysis' / 'environment'
out.mkdir(parents=True,exist_ok=True)
p = out / ('libusb-' + now.strftime('%Y%m%dT%H%M%S%z') + '.json')
p.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
print(p)
