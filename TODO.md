- DMA of multiple shifters isn't viable. Use a single DMA instead.
  Use multi-bit output. Use 2 timers per pin attached to those.
- Open question: The non-dma driver is quite elegant. Should we keep it?
- How to deal with the fact that flexio can only do 1,4,8,16 bit depth?
