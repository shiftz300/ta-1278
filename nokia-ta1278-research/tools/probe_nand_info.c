/* For the already-loaded, hash-pinned Nokia215 donor FDL2.
 * Its command registration at 0x8010a7b2 maps 0x29 to 0x8010b0ce,
 * which returns 12 bytes (type 0xb8) using the read-only device-info path.
 * Selector 0x90000000 maps to 0x8010bbd0: concatenated logical FTL reads,
 * NOT physical raw NAND/OOB. Only a 4096-byte prefix is read here.
 */
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main
int main(int argc,char **argv) {
    if(argc!=2) ERR_EXIT("usage: probe_nand_info output_directory\n");
    if(libusb_init(NULL)<0) ERR_EXIT("libusb init failed\n");
    libusb_device_handle *d=libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);
    if(!d) ERR_EXIT("target absent\n");
    spdio_t *io=spdio_init(d,0);io->flags|=FLAGS_TRANSCODE;io->verbose=1;
    encode_msg(io,BSL_CMD_READ_NAND_BLOCK_INFO,NULL,0);send_msg(io);
    if(!recv_msg_timeout(io,5000))ERR_EXIT("info timeout\n");
    unsigned n=READ16_BE(io->raw_buf+2);
    if(recv_type(io)!=0xb8 || n!=12)ERR_EXIT("unexpected info response\n");
    char name[4096];snprintf(name,sizeof(name),"%s/nand-info.bin",argv[1]);
    FILE *f=fopen(name,"wb");if(!f)ERR_EXIT("output failed\n");
    if(fwrite(io->raw_buf+4,1,n,f)!=n)ERR_EXIT("write failed\n");fclose(f);
    DBG_LOG("NAND info u32 LE: %u %u %u\n",READ32_LE(io->raw_buf+4),READ32_LE(io->raw_buf+8),READ32_LE(io->raw_buf+12));
    snprintf(name,sizeof(name),"%s/logical-full-prefix.bin",argv[1]);
    if(dump_flash(io,0x90000000,0,4096,name,4096,0)!=4096)ERR_EXIT("prefix incomplete\n");
    libusb_release_interface(d,0);libusb_close(d);libusb_exit(NULL);return 0;
}
