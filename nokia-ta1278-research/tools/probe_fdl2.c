/* Fixed-scope RAM FDL2 loader and NAND header probe.
 * Reuses the reviewed upstream transport; no flash-writing CLI is exposed.
 * Must start with the known custom FDL1 already running.
 */
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main

int main(int argc, char **argv) {
    if (argc != 4) ERR_EXIT("usage: probe_fdl2 fdl2.bin original_bootrom.bin output_directory\n");
    unsigned char expected[256], actual[256], head[512];
    FILE *f = fopen(argv[2], "rb");
    if (!f || fread(expected, 1, sizeof(expected), f) != sizeof(expected))
        ERR_EXIT("cannot read original ROM prefix\n");
    fclose(f);
    f = fopen(argv[1], "rb");
    if (!f || fread(head, 1, sizeof(head), f) != sizeof(head))
        ERR_EXIT("cannot read FDL2 header\n");
    fclose(f);
    if (READ32_LE(head) != 0x42544844 || READ32_LE(head+4) != 1)
        ERR_EXIT("expected DHTB v1 loader\n");
    int ret = libusb_init(NULL);
    if (ret < 0) ERR_EXIT("libusb_init failed\n");
    libusb_device_handle *dev = libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);
    if (!dev) ERR_EXIT("target not connected\n");
    spdio_t *io = spdio_init(dev, 0);
    io->flags |= FLAGS_TRANSCODE;
    io->flags &= ~FLAGS_CRC16;
    io->verbose = 1;
    if (read_flash(io,0xffff0000,0,256,actual,NULL,256) != 256 ||
        memcmp(expected,actual,256)) ERR_EXIT("known FDL1/ROM validation failed; no upload\n");
    DBG_LOG("Verified existing custom FDL1 against saved ROM prefix.\n");
    send_file(io,argv[1],0x80100000,1,528,0,0);
    encode_msg(io,BSL_CMD_EXEC_DATA,NULL,0);
    send_msg(io);
    if (!recv_msg_timeout(io,15000)) ERR_EXIT("FDL2 execute timeout\n");
    ret=recv_type(io);
    DBG_LOG("FDL2 execute response: 0x%04x\n",ret);
    if (ret!=BSL_REP_ACK && ret!=BSL_REP_INCOMPATIBLE_PARTITION)
        ERR_EXIT("FDL2 refused; stopping\n");
    /* Selectors are present in this PAC's directory/XML and upstream reader.
     * They are logical component selectors, not NAND physical offsets.
     */
    for (unsigned i=1;i<=4;i++) {
        char path[4096];
        if (snprintf(path,sizeof(path),"%s/component-%u-header.bin",argv[3],i)>=(int)sizeof(path))
            ERR_EXIT("path too long\n");
        unsigned n=read_flash(io,0x80000000u+i,0,512,head,NULL,512);
        if(n!=512) ERR_EXIT("component %u header incomplete, stopping\n",i);
        f=fopen(path,"wb");
        if(!f || fwrite(head,1,512,f)!=512) ERR_EXIT("cannot save header\n");
        fclose(f);
        DBG_LOG("Saved component %u header: %u bytes\n",i,n);
    }
    libusb_release_interface(dev,0);
    libusb_close(dev);
    libusb_exit(NULL);
    return 0;
}
