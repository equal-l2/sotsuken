Vue.component('var-list', {
    props: ['name', 'value'],
    data: function() {
        return Array.isArray(this.value) ? {
            val_array: this.value,
            val_str: null
        } : {
            val_array: null,
            val_str: this.value
        }
    },

    //FIXME: remove "v-else-if"
    //       "unresolved" and "__*" should have be removed by the backend.
    template: `
    <ul>
        <li v-if="val_str==null">
        {{name}}
        <var-list
            v-for="v in val_array"
            :name="v.name"
            :value="v.value"
            :key="JSON.stringify(name)+':'+JSON.stringify(v)">
        </var-list>
        </li>
        <li v-else-if="!(val_str=='unresolved'||name.startsWith('__'))">{{name}}:{{val_str}}</li>
    </ul>
    `,
})

let app = new Vue({
    el: "#app",

    data: {
        step: 0,
        trace: null,
    },

    computed: {
        line: function() {
            if (this.trace != null) {
                return this.trace.steps[this.step].loc.lineNumber;
            } else {
                return 0;
            }
        },
        vars: function() {
            if (this.trace != null) {
                return this.trace.steps[this.step].vars;
            } else {
                return null;
            }
        },
        step_max: function() {
            if (this.trace != null) {
                return this.trace.steps.length;
            } else {
                return 0;
            }
        }
    },

    methods: {
        readSrc: function(evn) {
            let f = evn.target.files[0];
            let rdr = new FileReader()
            rdr.readAsText(f);
            rdr.addEventListener("load", e => {
                try {
                    app.trace = JSON.parse(rdr.result);
                } catch (e) {
                    alert("Invalid file, cannot parse as JSON");
                    return;
                }
                this.step = 0;
            });
        }
    }
});
