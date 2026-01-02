import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  connect() {
    this.element.innerText = this.element.innerText.replaceAll("$$BASE_URL$$", window.location.origin);
    this.element.innerText = this.element.innerText.replaceAll("$$BASE_HOST$$", window.location.host);
  }
}