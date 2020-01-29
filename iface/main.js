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
        step_max: function() {
            if (this.trace != null) {
                return this.trace.steps.length;
            } else {
                return 0;
            }
        }
    },

    updated: function() {
        let var_cont = document.getElementById("var-cont");

        var_cont.innerHTML = "";
        for ([k, v] of Object.entries(app.trace.steps[app.step].vars)) {
            let li = document.createElement("li");
            let ul_nest = document.createElement("ul");
            var_cont.appendChild(li);
            li.appendChild(document.createTextNode(k));
            li.appendChild(ul_nest);
            for (c of v) {
                let li_nest = document.createElement("li");
                li_nest.appendChild(document.createTextNode(c.name + ":" + c.value.value));
                ul_nest.appendChild(li_nest);
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
