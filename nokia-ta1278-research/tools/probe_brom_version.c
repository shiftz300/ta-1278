/* Limited Spreadtrum BootROM version probe, based on reviewed spd_dump.c.
 * No loader upload, CONNECT, EXEC, flash read/write, erase or repartition.
 * Opens only VID:PID 1782:4d00; claims vendor interface with one bulk pair.
 */
#include <libusb-1.0/libusb.h>
#include <stdio.h>
#include <stdint.h>
#include <time.h>
#include <unistd.h>

static unsigned crc16(const unsigned char *p, size_t n) {
    unsigned c=0;
    while(n--) { c ^= (unsigned)*p++ << 8;
        for(int i=0;i<8;i++) c=(c<<1)^((c&0x8000)?0x1021:0);
        c &= 0xffff;
    }
    return c;
}
int main(void) {
    libusb_context *ctx=NULL;
    libusb_device_handle *h=NULL;
    struct libusb_config_descriptor *cfg=NULL;
    int rc=libusb_init(&ctx), iface=-1, epin=-1, epout=-1, claimed=0, result=1;
    setvbuf(stdout,NULL,_IOLBF,0);
    if(rc<0) { printf("init_error=%s\n",libusb_error_name(rc));return 1; }
    puts("waiting_for=1782:4d00; timeout_seconds=180; allowed_payload=CHECK_BAUD_only");
    time_t deadline=time(NULL)+180;
    while(time(NULL)<deadline) {
        h=libusb_open_device_with_vid_pid(ctx,0x1782,0x4d00);
        if(h) break;
        usleep(100000);
    }
    if(!h) { puts("open_failed_or_wait_expired");goto done; }
    puts("device_opened=1782:4d00");
    rc=libusb_get_config_descriptor(libusb_get_device(h),0,&cfg);
    if(rc<0) { printf("config_error=%s\n",libusb_error_name(rc));goto done; }
    for(int i=0;i<cfg->bNumInterfaces;i++) {
        const struct libusb_interface *it=&cfg->interface[i];
        if(it->num_altsetting<1) continue;
        const struct libusb_interface_descriptor *a=&it->altsetting[0];
        if(a->bInterfaceClass!=0xff || a->bAlternateSetting!=0) continue;
        int in=-1,out=-1,nb=0;
        for(int j=0;j<a->bNumEndpoints;j++) {
            const struct libusb_endpoint_descriptor *e=&a->endpoint[j];
            if((e->bmAttributes&3)!=LIBUSB_TRANSFER_TYPE_BULK) continue;
            nb++;
            if(e->bEndpointAddress&0x80) in=e->bEndpointAddress;else out=e->bEndpointAddress;
        }
        if(nb==2 && in>=0 && out>=0) {iface=a->bInterfaceNumber;epin=in;epout=out;break;}
    }
    if(iface<0) {puts("unexpected_interface_layout");goto done;}
    printf("interface=%d bulk_in=0x%02x bulk_out=0x%02x\n",iface,epin,epout);
    rc=libusb_claim_interface(h,iface);
    if(rc<0) {printf("claim_error=%s\n",libusb_error_name(rc));goto done;}
    claimed=1;
    /* Same interface initialization used before CHECK_BAUD by upstream.
     * USB class SET_CONTROL_LINE_STATE; no data stage and no flash command. */
    rc=libusb_control_transfer(h,0x21,34,0x601,iface,NULL,0,1000);
    printf("control_init_result=%d\n",rc);
    if(rc<0) goto done;
    unsigned char sync=0x7e,rx[4096],plain[4096];int sent=0,got=0;
    rc=libusb_bulk_transfer(h,epout,&sync,1,&sent,1000);
    printf("check_baud_result=%d sent=%d\n",rc,sent);
    if(rc<0 || sent!=1) goto done;
    rc=libusb_bulk_transfer(h,epin,rx,sizeof(rx),&got,1500);
    printf("receive_result=%d received=%d\n",rc,got);
    if(rc<0 || got<8 || rx[0]!=0x7e || rx[got-1]!=0x7e) goto done;
    size_t n=0;
    for(int i=1;i<got-1;i++) {
        unsigned char v=rx[i];
        if(v==0x7d) {if(++i>=got-1) goto done;v=rx[i]^0x20;}
        plain[n++]=v;
    }
    if(n<6) goto done;
    unsigned type=(plain[0]<<8)|plain[1],len=(plain[2]<<8)|plain[3];
    printf("reply_type=0x%04x payload_bytes=%u\n",type,len);
    if(n!=(size_t)len+6 || type!=0x81) goto done;
    unsigned expected=(plain[n-2]<<8)|plain[n-1],actual=crc16(plain,n-2);
    printf("crc_valid=%s\n",expected==actual?"true":"false");
    if(expected!=actual || len>128) goto done;
    printf("bootrom_version=");
    for(unsigned i=0;i<len;i++) {
        unsigned char v=plain[4+i];
        if(v==0) printf("\\0");else if(v>=32&&v<127) putchar(v);else printf("\\x%02x",v);
    }
    putchar('\n');result=0;
done:
    if(claimed) libusb_release_interface(h,iface);
    if(cfg) libusb_free_config_descriptor(cfg);
    if(h) libusb_close(h);
    libusb_exit(ctx);
    printf("probe_exit=%d\n",result);
    return result;
}
