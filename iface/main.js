let app = new Vue({
    el: "#app",

    data: {
        cnt: 0,
        max_cnt: 0,
        line: 0,
        trace_content: null,
    },

    methods: {
        nextStep: () => {
            app.cnt += 1;
            move_step();
        },
        prevStep: () => {
            app.cnt -= 1;
            move_step();
        },
        readSrc: evn => {
            let f = evn.target.files[0];
            let rdr = new FileReader()
            rdr.readAsText(f);
            rdr.addEventListener("load", e => {
                try {
                    app.trace_content = JSON.parse(rdr.result);
                } catch (e) {
                    alert("Invalid file, cannot parse as JSON");
                    return;
                }
                init_src_view();
            });
        }
    }
});

function init_src_view() {
    let src_cont = document.getElementById("src-cont");
    src_cont.innerHTML = ""
    app.cnt = 0;
    for (s of app.trace_content.src) {
        let li = document.createElement("li");
        li.appendChild(document.createTextNode(s));
        src_cont.appendChild(li);
    }
    app.max_cnt = app.trace_content.steps.length;
    move_step()
}

function move_step() {
    let var_cont = document.getElementById("var-cont");
    let new_line = app.trace_content.steps[app.cnt].loc.lineNumber;
    let src_cont = document.getElementById("src-cont");
    src_cont.childNodes[app.line].className = src_cont.childNodes[app.line].className.replace(/selected/, "");
    src_cont.childNodes[new_line].className = src_cont.childNodes[new_line].className + " selected";
    app.line = new_line;

    var_cont.innerHTML = "";
    let vars = app.trace_content.steps[app.cnt].vars;
    for ([k, v] of Object.entries(vars)) {
        let li = document.createElement("li");
        let ul_nest = document.createElement("ul");
        var_cont.appendChild(li);
        li.appendChild(document.createTextNode(k));
        li.appendChild(ul_nest);
        for (c of v) {
            let li_nest = document.createElement("li");
            //console.log(c.name);
            li_nest.appendChild(document.createTextNode(c.name + ":" + c.value.value));
            ul_nest.appendChild(li_nest);
        }
    }
}
