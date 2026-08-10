<script lang="ts">
  interface Props {
    slide1: any;
    slide2: any;
  }

  let { slide1, slide2 }: Props = $props();

  let activeIndex = $state(0);
  let isSwiping = $state(false);
  let startX = $state(0);
  let startY = $state(0);
  let dragOffset = $state(0);
  let isScrolling = $state(false);

  function handleTouchStart(e: TouchEvent) {
    startX = e.touches[0].clientX;
    startY = e.touches[0].clientY;
    isSwiping = true;
    isScrolling = false;
    dragOffset = 0;
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isSwiping) return;
    
    const currentX = e.touches[0].clientX;
    const currentY = e.touches[0].clientY;
    const deltaX = currentX - startX;
    const deltaY = currentY - startY;

    if (dragOffset === 0 && !isScrolling) {
      if (Math.abs(deltaY) > Math.abs(deltaX)) {
        isScrolling = true;
        isSwiping = false;
        return;
      }
    }

    if (isScrolling) return;

    dragOffset = deltaX;
  }

  function handleTouchEnd(e: TouchEvent) {
    if (!isSwiping) return;
    isSwiping = false;

    if (dragOffset < -40 && activeIndex === 0) {
      activeIndex = 1;
    } else if (dragOffset > 40 && activeIndex === 1) {
      activeIndex = 0;
    }
    dragOffset = 0;
  }
</script>

<div class="carousel-container">
  <div 
    class="carousel-track" 
    class:animating={!isSwiping}
    style="transform: translateX(calc({-activeIndex * 50}% + {dragOffset}px));"
    ontouchstart={handleTouchStart}
    ontouchmove={handleTouchMove}
    ontouchend={handleTouchEnd}
  >
    <div class="slide">
      {@render slide1()}
    </div>
    <div class="slide">
      {@render slide2()}
    </div>
  </div>
  
  <div class="dots">
    <div class="dot" class:active={activeIndex === 0}></div>
    <div class="dot" class:active={activeIndex === 1}></div>
  </div>
</div>

<style>
  .carousel-container {
    width: 100%;
    overflow: hidden;
    position: relative;
    padding-bottom: 20px;
  }

  .carousel-track {
    display: flex;
    width: 200%;
    touch-action: pan-y;
  }

  .carousel-track.animating {
    transition: transform 0.3s cubic-bezier(0.25, 1, 0.5, 1);
  }

  .slide {
    width: 50%;
    flex-shrink: 0;
  }

  .dots {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    gap: 8px;
    pointer-events: none;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.3);
    transition: background-color 0.2s ease;
  }

  .dot.active {
    background: #fff;
  }
</style>
