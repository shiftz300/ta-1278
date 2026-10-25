"""libusb adapter: no reset, driver detach or set-configuration.

One explicit native UCOM open/close control pair is supported after static
verification of the 17.00.17.02 kernel; never a generic control-request API.
"""
import ctypes as C
import ctypes.util


class Device(C.Structure):
    _fields_ = [('length', C.c_uint8), ('kind', C.c_uint8), ('usb', C.c_uint16),
                ('cls', C.c_uint8), ('sub', C.c_uint8), ('proto', C.c_uint8),
                ('packet', C.c_uint8), ('vid', C.c_uint16), ('pid', C.c_uint16),
                ('release', C.c_uint16), ('maker', C.c_uint8), ('product', C.c_uint8),
                ('serial', C.c_uint8), ('configs', C.c_uint8)]


class Endpoint(C.Structure):
    _fields_ = [('length', C.c_uint8), ('kind', C.c_uint8), ('address', C.c_uint8),
                ('attributes', C.c_uint8), ('packet', C.c_uint16), ('interval', C.c_uint8),
                ('refresh', C.c_uint8), ('sync', C.c_uint8), ('extra', C.c_void_p),
                ('extra_length', C.c_int)]


class Alternate(C.Structure):
    _fields_ = [('length', C.c_uint8), ('kind', C.c_uint8), ('number', C.c_uint8),
                ('alternate', C.c_uint8), ('count', C.c_uint8), ('cls', C.c_uint8),
                ('sub', C.c_uint8), ('proto', C.c_uint8), ('string', C.c_uint8),
                ('endpoints', C.POINTER(Endpoint)), ('extra', C.c_void_p),
                ('extra_length', C.c_int)]


class Interface(C.Structure):
    _fields_ = [('alternates', C.POINTER(Alternate)), ('count', C.c_int)]


class Config(C.Structure):
    _fields_ = [('length', C.c_uint8), ('kind', C.c_uint8), ('total', C.c_uint16),
                ('count', C.c_uint8), ('value', C.c_uint8), ('string', C.c_uint8),
                ('attributes', C.c_uint8), ('power', C.c_uint8),
                ('interfaces', C.POINTER(Interface)), ('extra', C.c_void_p),
                ('extra_length', C.c_int)]


class USB:
    def __init__(self):
        path = ctypes.util.find_library('usb-1.0') or '/opt/homebrew/opt/libusb/lib/libusb-1.0.dylib'
        self.lib = C.CDLL(path)
        specs = {
            'init': ([C.POINTER(C.c_void_p)], C.c_int),
            'exit': ([C.c_void_p], None),
            'get_device_list': ([C.c_void_p, C.POINTER(C.POINTER(C.c_void_p))], C.c_ssize_t),
            'free_device_list': ([C.POINTER(C.c_void_p), C.c_int], None),
            'get_device_descriptor': ([C.c_void_p, C.POINTER(Device)], C.c_int),
            'get_config_descriptor': ([C.c_void_p, C.c_uint8, C.POINTER(C.POINTER(Config))], C.c_int),
            'free_config_descriptor': ([C.POINTER(Config)], None),
            'get_bus_number': ([C.c_void_p], C.c_uint8),
            'get_device_address': ([C.c_void_p], C.c_uint8),
            'open': ([C.c_void_p, C.POINTER(C.c_void_p)], C.c_int),
            'close': ([C.c_void_p], None),
            'claim_interface': ([C.c_void_p, C.c_int], C.c_int),
            'release_interface': ([C.c_void_p, C.c_int], C.c_int),
            'bulk_transfer': ([C.c_void_p, C.c_ubyte, C.c_void_p, C.c_int, C.POINTER(C.c_int), C.c_uint], C.c_int),
            'control_transfer': ([C.c_void_p, C.c_uint8, C.c_uint8, C.c_uint16,
                                  C.c_uint16, C.c_void_p, C.c_uint16, C.c_uint], C.c_int),
        }
        for name, (args, ret) in specs.items():
            fn = getattr(self.lib, 'libusb_'+name)
            fn.argtypes, fn.restype = args, ret
        self.ctx = C.c_void_p()
        self.handle = None
        self.interface = None
        self.ucom_opened = False
        self.check(self.lib.libusb_init(C.byref(self.ctx)), 'init')

    @staticmethod
    def check(code, action):
        if code < 0:
            raise OSError(f'libusb {action}: {code}')

    def _devices(self):
        devices = C.POINTER(C.c_void_p)()
        n = self.lib.libusb_get_device_list(self.ctx, C.byref(devices))
        self.check(n, 'enumerate')
        return devices, n

    def inventory(self):
        devices, n = self._devices()
        result = []
        try:
            for i in range(n):
                d = Device()
                self.check(self.lib.libusb_get_device_descriptor(devices[i], C.byref(d)), 'descriptor')
                if d.vid not in (0x1782, 0x2e04):
                    continue
                row = {'vid': d.vid, 'pid': d.pid, 'bcdDevice': d.release,
                       'bus': self.lib.libusb_get_bus_number(devices[i]),
                       'address': self.lib.libusb_get_device_address(devices[i]), 'interfaces': []}
                cfg = C.POINTER(Config)()
                rc = self.lib.libusb_get_config_descriptor(devices[i], 0, C.byref(cfg))
                if rc < 0:
                    row['config_error'] = rc
                else:
                    try:
                        for j in range(cfg.contents.count):
                            iface = cfg.contents.interfaces[j]
                            for k in range(iface.count):
                                a = iface.alternates[k]
                                row['interfaces'].append({'number': a.number, 'alternate': a.alternate,
                                    'class': a.cls, 'subclass': a.sub, 'protocol': a.proto,
                                    'endpoints': [{'address': a.endpoints[t].address,
                                                  'attributes': a.endpoints[t].attributes,
                                                  'max_packet': a.endpoints[t].packet} for t in range(a.count)]})
                    finally:
                        self.lib.libusb_free_config_descriptor(cfg)
                result.append(row)
        finally:
            self.lib.libusb_free_device_list(devices, 1)
        return result

    def connect(self, profile):
        # Exact profile selected from fresh descriptors. No automatic guessing
        # between a composite device's modem, AT and diagnostic interfaces.
        matches = [r for r in self.inventory() if (r['vid'], r['pid']) == (profile['vid'], profile['pid'])]
        if len(matches) != 1:
            raise OSError('Expected exactly one matching phone')
        row = matches[0]
        if row['pid'] == 0x4d00 or row['vid'] not in (0x1782, 0x2e04):
            raise ValueError('Boot/acquisition or unknown device is blocked')
        candidates = [a for a in row['interfaces'] if a['number'] == profile['interface'] and a['alternate'] == 0]
        if len(candidates) != 1 or candidates[0]['class'] != 0xff:
            raise ValueError('Unverified interface; only explicit vendor bulk pairs supported')
        bulk = [e['address'] for e in candidates[0]['endpoints'] if e['attributes'] & 3 == 2]
        if sorted(bulk) != sorted([profile['in'], profile['out']]) or not profile['in'] & 0x80 or profile['out'] & 0x80:
            raise ValueError('USB endpoint layout changed')
        devices, n = self._devices()
        try:
            for i in range(n):
                if (self.lib.libusb_get_bus_number(devices[i]), self.lib.libusb_get_device_address(devices[i])) == (row['bus'], row['address']):
                    h = C.c_void_p()
                    self.check(self.lib.libusb_open(devices[i], C.byref(h)), 'open')
                    self.handle = h
                    self.interface = profile['interface']
                    try:
                        self.check(self.lib.libusb_claim_interface(h, self.interface), 'claim (no driver detach)')
                    except Exception:
                        self.lib.libusb_close(h)
                        self.handle = None
                        raise
                    self.ep_in, self.ep_out = profile['in'], profile['out']
                    if profile.get('native_ucom_open') is True:
                        self.open_native_ucom(profile)
                    return
            raise OSError('Phone disconnected before open')
        finally:
            self.lib.libusb_free_device_list(devices, 1)

    def open_native_ucom(self, profile):
        if (profile['vid'], profile['pid'], profile['interface'],
                profile['in'], profile['out'], profile.get('expected_firmware')) != (
                0x2e04, 0x19, 1, 0x83, 4, '17.00.17.02'):
            raise ValueError('Native UCOM open is restricted to the reviewed profile')
        # kernel file 0x177bc4..0x177cbc: request 0x22, high byte of
        # wValue=OUT endpoint 4, low byte=1 -> READY, clear RAM FIFOs,
        # start receive. wIndex is unused by that native handler.
        self.check(self.lib.libusb_control_transfer(
            self.handle, 0x21, 0x22, 0x0401, self.interface, None, 0, 2000),
            'native UCOM open')
        self.ucom_opened = True

    def write(self, data):
        buffer = C.create_string_buffer(data)
        count = C.c_int()
        self.check(self.lib.libusb_bulk_transfer(self.handle, self.ep_out, buffer, len(data), C.byref(count), 2000), 'write')
        if count.value != len(data):
            raise OSError('Short USB write; never retry a key')

    def read(self, timeout_ms):
        buffer = C.create_string_buffer(65536)
        count = C.c_int()
        rc = self.lib.libusb_bulk_transfer(self.handle, self.ep_in, buffer, len(buffer), C.byref(count), timeout_ms)
        if rc == -7:  # timeout may still return partial data
            return buffer.raw[:count.value]
        self.check(rc, 'read')
        return buffer.raw[:count.value]

    def close(self):
        if self.handle:
            if self.ucom_opened:
                # Native close uses the IN endpoint number (3), not OUT (4).
                self.lib.libusb_control_transfer(
                    self.handle, 0x21, 0x22, 0x0300, self.interface, None, 0, 1000)
                self.ucom_opened = False
            self.lib.libusb_release_interface(self.handle, self.interface)
            self.lib.libusb_close(self.handle)
            self.handle = None
        if self.ctx:
            self.lib.libusb_exit(self.ctx)
            self.ctx = None
