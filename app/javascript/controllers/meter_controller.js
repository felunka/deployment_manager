import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  connect() {
    let value = parseInt(this.element.dataset.value);
    this.element.style.setProperty("--value", value);

    let style = window.getComputedStyle(document.body);
    if(value < 70) {
      this.element.style.setProperty("--color", style.getPropertyValue("--bs-green"));
    } else if(value < 90) {
      this.element.style.setProperty("--color", style.getPropertyValue("--bs-orange"));
    } else {
      this.element.style.setProperty("--color", style.getPropertyValue("--bs-red"));
    }

    let valueDisplay = document.createElement("span");
    valueDisplay.innerText = `${value}%`;
    this.element.appendChild(valueDisplay);
  }
}
