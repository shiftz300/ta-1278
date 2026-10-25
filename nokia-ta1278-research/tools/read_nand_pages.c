/* Only for the verified debug FDL2 + hook SHA3918bf66... .
 * A120 calls pinned native NAND READ with ECC flag 0; no program/erase.
 * Save all physical pages including spare. Stop on any native read error;
 * never replace failed pages with FF. Layout interpretation is recorded
 * separately; this is the native controller view, not a claim of bit perfection.
 */
#define main upstream_unused_main
#include "upstream-reference/spreadtrum_flash/spd_dump.c"
#undef main
static FILE *create(const char *dir,const char *name){
    char p[4096];snprintf(p,sizeof(p),"%s/%s",dir,name);FILE *f=fopen(p,"wx");if(!f)ERR_EXIT("output exists or cannot create\n");return f;
}
static int page(spdio_t *io,unsigned sel,unsigned p,unsigned char *b,unsigned size){
    unsigned n=read_flash(io,sel,p*2048,size,b,NULL,size);
    if(n!=size || READ32_LE(b)!=0x44524e50 || READ32_LE(b+4)!=p*2048)ERR_EXIT("invalid page response\n");
    return READ32_LE(b+8);
}
int main(int argc,char **argv){
    if(argc!=3 || (strcmp(argv[1],"probe") && strcmp(argv[1],"full")))ERR_EXIT("usage: read_nand_pages probe|full output-dir\n");
    if(libusb_init(NULL)<0)ERR_EXIT("libusb init failed\n");
    libusb_device_handle *d=libusb_open_device_with_vid_pid(NULL,0x1782,0x4d00);if(!d)ERR_EXIT("target absent\n");
    spdio_t *io=spdio_init(d,0);io->flags|=FLAGS_TRANSCODE;io->timeout=5000;
    unsigned char b[2192];
    if(read_flash(io,0xa0000000,0x8010c55c,8,b,NULL,8)!=8 || memcmp(b,"\xdf\xf8\x00\xf0\x01\x00\x00\x83",8))ERR_EXIT("debug hook not present\n");
    FILE *status=create(argv[2],"page-status.jsonl");
    if(!strcmp(argv[1],"probe")){
        unsigned pages[]={0,1,64,65535};
        for(unsigned i=0;i<4;i++)for(unsigned rep=1;rep<=2;rep++){
            int rc=page(io,0xa1200000,pages[i],b,2192);char name[100];
            snprintf(name,sizeof(name),"page-%05u-%u.record",pages[i],rep);
            FILE *f=create(argv[2],name);fwrite(b,1,2192,f);fclose(f);
            fprintf(status,"{\"page\":%u,\"pass\":%u,\"native_status\":%d,\"spare_bytes\":%u}\n",pages[i],rep,rc,READ32_LE(b+12));fflush(status);
            printf("page %u pass %u: status=%d spare=%u\n",pages[i],rep,rc,READ32_LE(b+12));fflush(stdout);
            if(rc || READ32_LE(b+12)!=64)ERR_EXIT("native read/geometry error; no full dump\n");
        }
    }else{
        FILE *main=create(argv[2],"nand-main.partial"),*spare=create(argv[2],"nand-spare.partial");
        for(unsigned p=0;p<65536;p++){
            int rc=page(io,0xa1200000,p,b,2192);
            if(rc || READ32_LE(b+12)!=64){fprintf(status,"{\"failed_page\":%u,\"native_status\":%d}\n",p,rc);fflush(status);ERR_EXIT("native read failed at page %u\n",p);}
            if(fwrite(b+16,1,2048,main)!=2048 || fwrite(b+2064,1,64,spare)!=64)ERR_EXIT("disk write failed\n");
            if((p&1023)==1023){fflush(main);fflush(spare);fprintf(status,"{\"through_page\":%u,\"all_native_status_zero\":true}\n",p);fflush(status);printf("%u / 65536 pages\n",p+1);fflush(stdout);}
        }
        fclose(main);fclose(spare);
    }
    fclose(status);libusb_release_interface(d,0);libusb_close(d);libusb_exit(NULL);return 0;
}
