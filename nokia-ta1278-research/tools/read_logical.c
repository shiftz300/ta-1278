/* Bounded, resumable READ_FLASH only. Never prints payload bytes.
 * Drain an interrupted predecessor's USB IN response before sending a read.
 * Resume is only validated by the independent second full read in Python.
 */
#include <signal.h>
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main
static volatile sig_atomic_t stopped;
static void stop(int sig){(void)sig;stopped=1;}
int main(int argc,char **argv){
    if(argc!=4)ERR_EXIT("usage: read_logical selector bounded-size output.partial\n");
    unsigned id=strtoul(argv[1],0,0),size=strtoul(argv[2],0,0);
    if(!((id>=0x80000001 && id<=0x80000008)||(id>=0x90000001 && id<=0x9000001d)) || !size || size>0x8000000)ERR_EXIT("invalid bound\n");
    FILE *f=fopen(argv[3],"ab+");if(!f)ERR_EXIT("open failed\n");fseek(f,0,SEEK_END);long old=ftell(f);
    if(old<0 || (unsigned long)old>size || ((unsigned)old&511))ERR_EXIT("invalid partial length\n");
    if((unsigned)old==size){fclose(f);return 0;}
    if(libusb_init(NULL)<0)ERR_EXIT("libusb init failed\n");
    libusb_device_handle *d=libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);
    if(!d)ERR_EXIT("target absent\n");
    spdio_t *io=spdio_init(d,0);io->flags|=FLAGS_TRANSCODE;io->timeout=5000;
    unsigned char discard[65536];int got;
    for(unsigned i=0;i<8;i++) {
        int rc=libusb_bulk_transfer(d,io->endp_in,discard,sizeof(discard),&got,250);
        if(rc==LIBUSB_ERROR_TIMEOUT && !got)break;
        if(rc && rc!=LIBUSB_ERROR_TIMEOUT)ERR_EXIT("USB drain failed\n");
        if(i==7)ERR_EXIT("USB never idle\n");
    }
    signal(SIGINT,stop);signal(SIGTERM,stop);
    unsigned off=old;
    while(off<size && !stopped){
        unsigned n=size-off;if(n>0x4000)n=0x4000;
        unsigned read=read_flash(io,id,off,n,NULL,f,n);
        fflush(f);if(read!=n)ERR_EXIT("incomplete at 0x%x\n",off);off+=n;
        if(!(off&0xfffff)){printf("%08x: %u/%u\n",id,off,size);fflush(stdout);}
    }
    fclose(f);libusb_release_interface(d,0);libusb_close(d);libusb_exit(NULL);
    return off==size?0:2;
}
