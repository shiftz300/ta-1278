/* Read-only boundary discovery for the hash-pinned, already-running FDL2.
 * c55c checks offset+length against FTL sector_size*sector_count before read.
 * 90000001 is excluded: its out-of-range path synthesizes FF bytes.
 * Bounds are provisional until a complete sequential read succeeds.
 * Only BSL_CMD_READ_FLASH is sent. Payload bytes are never printed/saved.
 */
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main
static FILE *trace;
static int probe(spdio_t *io, uint32_t id, uint32_t off) {
    uint32_t data[3];
    WRITE32_BE(data,id);WRITE32_BE(data+1,512);WRITE32_BE(data+2,off);
    encode_msg(io,BSL_CMD_READ_FLASH,data,12);send_msg(io);
    if(!recv_msg_timeout(io,5000)) ERR_EXIT("USB timeout; stopping\n");
    int type=recv_type(io); unsigned n=READ16_BE(io->raw_buf+2);
    fprintf(trace,"{\"selector\":\"0x%08x\",\"offset\":%u,\"response\":%d,\"length\":%u}\n",id,off,type,n);fflush(trace);
    if(type==BSL_REP_READ_FLASH && n==512)return 1;
    if(n==0 && (type==0x84 || type==0x89 || type==0x8a))return 0;
    ERR_EXIT("unexpected response 0x%x length %u; stopping\n",type,n);
}
int main(int argc,char **argv) {
    if(argc!=3) ERR_EXIT("usage: map_logical_parts results.jsonl trace.jsonl\n");
    FILE *f=fopen(argv[1],"wx");trace=fopen(argv[2],"wx");
    if(!f || !trace)ERR_EXIT("outputs already exist or cannot create\n");
    if(libusb_init(NULL)<0)ERR_EXIT("libusb init failed\n");
    libusb_device_handle *d=libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);
    if(!d)ERR_EXIT("target absent\n");
    spdio_t *io=spdio_init(d,0);io->flags|=FLAGS_TRANSCODE;
    for(unsigned k=0;k<35;k++) {
        uint32_t id=k<7?0x80000002+k:0x90000002+(k-7);
        if(!probe(io,id,0)){fprintf(f,"{\"selector\":\"0x%08x\",\"readable\":false}\n",id);fflush(f);continue;}
        uint32_t low=0,high=1;
        while(high<262144 && probe(io,id,high*512)){low=high;high*=2;}
        if(high==262144 && probe(io,id,high*512))ERR_EXIT("no bound within 128 MiB; stopping\n");
        while(high-low>1){uint32_t mid=low+(high-low)/2;if(probe(io,id,mid*512))low=mid;else high=mid;}
        fprintf(f,"{\"selector\":\"0x%08x\",\"readable\":true,\"provisional_bytes\":%u}\n",id,high*512);fflush(f);
        printf("%08x: %u bytes bounded at 512-byte resolution\n",id,high*512);fflush(stdout);
    }
    fclose(f);fclose(trace);libusb_release_interface(d,0);libusb_close(d);libusb_exit(NULL);return 0;
}
