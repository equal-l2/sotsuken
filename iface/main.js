let src_cont = document.getElementById("src-cont");
let var_cont = document.getElementById("var-cont");
let cnt = 0
let line = 0;
let trace_content;

function init_src_view() {
    src_cont.innerHTML = ""
    cnt = 0;
    for (s of trace_content.src) {
        let li = document.createElement("li");
        li.appendChild(document.createTextNode(s));
        src_cont.appendChild(li);
    }
    move_step()
}

function move_step() {
    let new_line = trace_content.steps[cnt].loc.lineNumber;
    src_cont.childNodes[line].className = src_cont.childNodes[line].className.replace(/selected/, "");
    src_cont.childNodes[new_line].className = src_cont.childNodes[new_line].className + " selected";
    line = new_line;

    var_cont.innerHTML = "";
    let vars = trace_content.steps[cnt].vars;
    for ([k, v] of Object.entries(vars)) {
        let li = document.createElement("li");
        let ul_nest = document.createElement("ul");
        var_cont.appendChild(li);
        li.appendChild(document.createTextNode(k));
        li.appendChild(ul_nest);
        for (c of v) {
            let li_nest = document.createElement("li");
            console.log(c.name);
            li_nest.appendChild(document.createTextNode(c.name + ":" + c.value.value));
            ul_nest.appendChild(li_nest);
        }
    }
}

{
    let trace_read = document.getElementById("src-read");
    trace_read.addEventListener("change", function(e) {
        let f = e.target.files[0];
        let rdr = new FileReader()
        rdr.readAsText(f);
        rdr.addEventListener("load", e => {
            try {
                trace_content = JSON.parse(rdr.result);
            } catch (e) {
                alert("Invalid file, cannot parse as JSON");
            }
            init_src_view();
        });
    });
}

{
    let next_button = document.getElementById("next");
    next_button.addEventListener("click", e => {
        cnt += 1;
        console.log(line);
        move_step();
    });
}

{
    let prev_button = document.getElementById("prev");
    prev_button.addEventListener("click", e => {
        cnt -= 1;
        console.log(line);
        move_step();
    });
}
