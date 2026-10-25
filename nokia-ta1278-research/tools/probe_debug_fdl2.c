/* Start only in known custom FDL1. Upload RAM-only extension and modified
 * donor FDL2, verify uploaded bytes, execute, then read fixed RAM metadata.
 * No physical NAND experiment is triggered by this program.
 */
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main
static void saved(const char *dir,const char *name,const void *p,unsigned n){
    char path[4096];snprintf(path,sizeof(path),"%s/%s",dir,name);
    FILE *f=fopen(path,"wb");if(!f || fwrite(p,1,n,f)!=n)ERR_EXIT("save failed\n");fclose(f);
}
int main(int argc,char **argv){
    if(argc!=5)ERR_EXIT("usage: probe_debug_fdl2 hook.bin patched-fdl2.bin bootrom.bin output-dir\n");
    unsigned char expected[256],actual[4096];
    FILE *f=fopen(argv[3],"rb");if(!f || fread(expected,1,256,f)!=256)ERR_EXIT("ROM input error\n");fclose(f);
    if(libusb_init(NULL)<0)ERR_EXIT("libusb init failed\n");
    libusb_device_handle *d=libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);
    if(!d)ERR_EXIT("target absent\n");
    spdio_t *io=spdio_init(d,0);io->flags|=FLAGS_TRANSCODE;io->verbose=1;
    if(read_flash(io,0xffff0000,0,256,actual,NULL,256)!=256 || memcmp(expected,actual,256))ERR_EXIT("not verified FDL1; no upload\n");
    size_t n;unsigned char *hook=loadfile(argv[1],&n,0);if(!hook || !n || n>4096)ERR_EXIT("bad hook\n");
    send_file(io,argv[1],0x83000000,1,528,0,0);
    /* Custom FDL1 ignores the offset word: use one bounded packet, not
     * FDL2's fixed-selector/increasing-offset loop, for RAM verification. */
    if(n>=2048)ERR_EXIT("hook too large for one FDL1 read\n");
    if(read_flash(io,0x83000000,0,n,actual,NULL,n)!=n || memcmp(hook,actual,n))ERR_EXIT("RAM hook verify failed\n");
    free(hook);send_file(io,argv[2],0x80100000,1,528,0,0);
    if(read_flash(io,0x8010c55c,0,8,actual,NULL,8)!=8 || memcmp(actual,"\xdf\xf8\x00\xf0\x01\x00\x00\x83",8))ERR_EXIT("patch readback failed\n");
    encode_msg(io,BSL_CMD_EXEC_DATA,NULL,0);send_msg(io);
    if(!recv_msg_timeout(io,15000) || recv_type(io)!=BSL_REP_ACK)ERR_EXIT("debug FDL2 execute failed\n");
    if(read_flash(io,0x80000001,0,512,actual,NULL,512)!=512 || memcmp(actual,"DHTB\x01\0\0\0",8))ERR_EXIT("original read regression\n");
    saved(argv[4],"boot0-header.bin",actual,512);
    if(read_flash(io,0xa0000000,0x80121498,64,actual,NULL,64)!=64)ERR_EXIT("RAM metadata read failed\n");
    saved(argv[4],"nand-globals.bin",actual,64);
    unsigned spec=READ32_LE(actual+0x10),dev=READ32_LE(actual+0x18);
    if(spec<0x80100000 || spec>0x802fff80 || dev<0x80100000 || dev>0x802fff80)ERR_EXIT("unexpected pointers\n");
    if(read_flash(io,0xa0000000,spec,128,actual,NULL,128)!=128)ERR_EXIT("spec read failed\n");
    saved(argv[4],"nand-spec.bin",actual,128);
    if(read_flash(io,0xa0000000,dev,128,actual,NULL,128)!=128)ERR_EXIT("device descriptor read failed\n");
    saved(argv[4],"nand-device.bin",actual,128);
    if(read_flash(io,0xa2000000,0x10000001,32,actual,NULL,32)!=32)ERR_EXIT("NV partition info failed\n");
    saved(argv[4],"nv-part-info.bin",actual,32);
    DBG_LOG("RAM debug metadata collected. No physical-page experiment yet.\n");
    libusb_release_interface(d,0);libusb_close(d);libusb_exit(NULL);return 0;
}
