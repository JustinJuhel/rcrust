/* STM32F401CCUx: 256K flash, 64K RAM */
/* Adjust if using a different variant (e.g. STM32F401RE: 512K flash, 96K RAM) */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 256K
    RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
