class cursors {
  constructor() {
    this.first = true;
    this.outer = document.getElementById("cursor-outer").style;
    this.effecter = document.getElementById("cursor-effect").style;
    this.scale = 0;
    this.opacity = 0;
    this.last = 0;
    this.moveIng = false;
    this.fadeIng = false;
    this.attention =
      "a,input,button,.typing-tip,.eval-arrow,.example-tag";
    this.effecter.transform = "translate(-50%, -50%) scale(0)";
    this.effecter.opacity = "1";
    window.addEventListener("mousemove", (mouse) => this.reset(mouse), {
      passive: true,
    });
    window.addEventListener("click", (mouse) => this.Aeffect(mouse), {
      passive: true,
    });
    this.pushHolders();
    const observer = new MutationObserver(this.pushHolders.bind(this));
    observer.observe(document, { childList: true, subtree: true });
  }
  move(timestamp) {
    if (this.now !== undefined) {
      let SX = this.outer.left,
        SY = this.outer.top;
      let preX = Number(SX.substring(0, SX.length - 2)),
        preY = Number(SY.substring(0, SY.length - 2));
      let delX = (this.now.x - preX) * 0.3,
        delY = (this.now.y - preY) * 0.3;
      preX += delX;
      preY += delY;
      this.outer.left = preX.toFixed(2) + "px";
      this.outer.top = preY.toFixed(2) + "px";
      if (Math.abs(delX) > 0.2 || Math.abs(delY) > 0.2) {
        while (timestamp - this.last < 10) this.last = timestamp;
        window.requestAnimationFrame(this.move.bind(this));
      } else {
        this.moveIng = false;
      }
    }
  }
  reset(mouse) {
    if (!this.moveIng) {
      this.moveIng = true;
      window.requestAnimationFrame(this.move.bind(this));
    }
    this.now = mouse;
    if (this.first) {
      this.first = false;
      this.outer.left = String(this.now.x) + "px";
      this.outer.top = String(this.now.y) + "px";
    }
  }
  Aeffect(mouse) {
    if (this.fadeIng == false) {
      let a = this;
      this.fadeIng = true;
      this.effecter.left = String(mouse.x) + "px";
      this.effecter.top = String(mouse.y) + "px";
      this.effecter.transition =
        "transform .5s cubic-bezier(0.22, 0.61, 0.21, 1), opacity .5s cubic-bezier(0.22, 0.61, 0.21, 1)";
      this.effecter.transform = "translate(-50%, -50%) scale(1)";
      this.effecter.opacity = "0";
      setTimeout(() => {
        this.fadeIng = false;
        this.effecter.transition = "";
        this.effecter.transform = "translate(-50%, -50%) scale(0)";
        this.effecter.opacity = "1";
      }, 500);
    }
  }
  hold() {
    this.outer.height = "24px";
    this.outer.width = "24px";
    this.outer.background = "rgba(255, 255, 255, 0.5)";
  }
  relax() {
    this.outer.height = "36px";
    this.outer.width = "36px";
    this.outer.background = "unset";
  }
  pushHolder(items) {
    items.forEach((item) => {
      if (!item.classList.contains("is--active")) {
        item.addEventListener("mouseover", () => this.hold(), {
          passive: true,
        });
        item.addEventListener("mouseout", () => this.relax(), {
          passive: true,
        });
      }
    });
  }
  pushHolders() {
    this.pushHolder(document.querySelectorAll(this.attention));
  }
}

const cursor = new cursors();
