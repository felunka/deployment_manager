import jQuery from "jquery"
import 'select2'

export default class DatagridEnumFilter {
  constructor(target, filterInputId) {
    this.target = target;
    this.filterInput = document.getElementById(filterInputId);
  }

  generateFilter() {
    let wrapper = document.createElement('div');
    wrapper.classList.add('filter-select2-wrapper');

    let input = this.filterInput.cloneNode(true);
    input.dataset.action = 'keydown->datagrid#inputKeyDown';
    
    wrapper.appendChild(input);

    $(input).select2({
      dropdownParent: $(wrapper)
    });

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
    let selectedValues = $(this.input).select2('data').map(element => element.id);
    for(let i = 0; i < this.filterInput.options.length; i++) {
      let option = this.filterInput.options[i];
      if(selectedValues.includes(option.value)) {
        option.selected = true;
      } else {
        option.selected = false;
      }
    }
    this.filterInput.closest('form').submit();
  }

  reset() {
    for(let i = 0; i < this.filterInput.options.length; i++) {
      this.filterInput.options[i].selected = false;
    }
    this.filterInput.closest('form').submit();
  }
};
