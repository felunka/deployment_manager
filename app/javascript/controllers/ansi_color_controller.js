import { Controller } from '@hotwired/stimulus'
import { AnsiUp } from 'ansi_up'

export default class extends Controller {
  connect() {
    let ansi_up = new AnsiUp();
    this.element.innerHTML = ansi_up.ansi_to_html(this.element.innerText);
  }
}