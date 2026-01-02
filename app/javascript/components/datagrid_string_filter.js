export default class DatagridStringFilter {
  constructor(target, filterInputId) {
    this.target = target;
    this.filterInput = document.getElementById(filterInputId);
  }

  generateFilter() {
    let wrapper = document.createElement('div');
    wrapper.classList.add('input-group');

    let input = document.createElement('input');
    input.classList.add('form-control');
    input.dataset.action = 'keydown->datagrid#inputKeyDown';
    input.type = 'text';
    input.placeholder = this.target.dataset.label;
    input.value = this.filterInput.value;
    wrapper.appendChild(input);

    this.input = input;

    let button = document.createElement('a');
    button.role = 'button';
    button.dataset.action = 'click->datagrid#searchButtonClick';
    button.classList.add('btn', 'btn-outline-primary');
    wrapper.appendChild(button);

    let icon = document.createElement('i');
    icon.classList.add('bi', 'bi-search');
    button.appendChild(icon);

    return wrapper;
  }

  search() {
    let filterValue = this.input.value;
    this.filterInput.value = filterValue;
    this.filterInput.closest('form').submit();
  }

  reset() {
    this.filterInput.value = '';
    this.filterInput.closest('form').submit();
  }
};
