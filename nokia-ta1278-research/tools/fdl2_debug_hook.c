/* RAM-only extension for exactly SHA256 3c6699d6...59e4adb7bd FDL2.
 * Called with READ_FLASH arguments (selector, offset, length, destination).
 * No program, erase, repartition, arbitrary execution or write API.
 * Native FTL and NAND READ entrypoints are pinned to the audited binary.
 * A100 is experimental physical main-area read, NOT raw OOB or proven ECC mode.
 */
typedef unsigned U; typedef unsigned short H; typedef unsigned char B;
#define FN(a,t) ((t)(a))
#define R32(a) (*(volatile U *)(a))
#define R16(a) (*(volatile H *)(a))
__attribute__((used)) int debug_read(U id,U off,U len,B *out) {
    if(id==0xa0000000) {
        if(off<0x80100000 || off>0x80300000 || len>0x4000 || len>0x80300000-off)return 6;
        for(U i=0;i<len;i++)out[i]=*(volatile B *)(off+i);
        return 0;
    }
    if(id==0xa2000000) {
        /* Query existing partition metadata: no change to persistent table. */
        if(len!=32 || off>0x1000001d)return 6;
        U handle=0,info[4]={0,0,0,0};
        int a=FN(0x8010cba7,int(*)(U,U,U*,U))(0,off,&handle,4),b=-1;
        if(!a){b=FN(0x8010cc8d,int(*)(U,U*))(handle,info);FN(0x8010ceab,int(*)(U))(handle);}
        U words[8]={0x514c5446,(U)a,(U)b,handle,info[0],info[1],info[2],info[3]};
        for(U i=0;i<32;i++)out[i]=((B*)words)[i];return 0;
    }
    if(id==0xa1000000 || id==0xa1100000 || id==0xa1200000) {
        U spec=R32(0x801214a8),dev=R32(0x801214b0);
        U combined=id==0xa1200000;
        if(len!=(combined?2192:2064) || off>=0x08000000 || (off&2047))return 6;
        if(spec<0x80100000 || spec>0x80300000-64 || dev<0x80100000 || dev>0x80300000-64)return 6;
        /* Refuse any unrecognized geometry. These fields are checked again
         * from actual RAM metadata before this route is used by the host. */
        if(R32(spec+8)!=1024 || R16(spec+0x14)!=1024 || R16(spec+0x18)!=2 ||
           R32(dev+8)!=2048 || R16(dev+0xc)!=1024 || R16(dev+0xe)!=64)return 6;
        B *buf=(B*)0x83010000;
        B *spare=(B*)0x83011000;
        U spare_bytes=R16(dev+0x26)*2;
        if(!spare_bytes || spare_bytes>128)return 6;
        for(U i=0;i<2048;i++)buf[i]=0x5a;
        for(U i=0;i<128;i++)spare[i]=0x5a;
        int rc=FN(0x8010949d,int(*)(U,U,U,U,U,B*,B*,U*,U))
            (dev,off>>17,(off>>11)&63,0,2,buf,spare,(U*)0,id==0xa1000000?1:0);
        U header[4]={0x44524e50,off,(U)rc,combined?spare_bytes:2048};
        for(U i=0;i<16;i++)out[i]=((B*)header)[i];
        for(U i=0;i<2048;i++)out[16+i]=buf[i];
        if(combined)for(U i=0;i<128;i++)out[2064+i]=spare[i];
        return 0;
    }
    return 1;
}
