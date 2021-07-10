var BORDER_CONSTANT = "5px solid rgb(165, 42, 42)";

function move_robot(robot, direction){

    let robot_el = $(".robot_field").filter( function (){ 
        let color = $(this).css("background-color").replace(/[^\d,]/g, '').split(',');
        return rgbToHex(color[0], color[1], color[2]) === robot;
    }).get(0);

    let row = parseInt($(robot_el).attr("row")), col = parseInt($(robot_el).attr("col"));

    let i_shift, j_shift

    if(direction === "left"){
        i_shift = 0;
        j_shift = -1;
    }
    else if(direction === "top"){
        i_shift = -1;
        j_shift = 0;
    }
    else if(direction === "right"){
        i_shift = 0;
        j_shift = 1;
    }
    else if(direction === "bottom"){
        i_shift = 1;
        j_shift = 0;
    }
    else{
        console.log("Invalid direction.");
        return 0;
    }
    let curr_row = row;
    let curr_col = col;

    let next = $("td[row=" + curr_row + "][col=" + curr_col + "]");

    while( next.css("border-" + direction) != BORDER_CONSTANT ){
        curr_row += i_shift
        curr_col += j_shift
        next = $("td[row=" + curr_row + "][col=" + curr_col + "]");

        if ( next.attr("class") === "robot_field" ){
            curr_row -= i_shift;
            curr_col -= j_shift;
            next = $("td[row=" + curr_row + "][col=" + curr_col + "]");
            break;
        }
            
    }

    // OVO IMENOVANJE (KAO I ZAMJENA BOJA) MORA IĆI OVIM REDOSLJEDOM
    // AKO NE IDE I U SLUCAJU KAD SE ROBOT ZABIJE U ZID ILI U DRUGOG
    // ROBOTA OSTAT ĆE NA MJESTU ALI VISE NECE BITI PREPOZNAT KAO ROBOT

    $(robot_el).attr("class", "board_field");
    next.attr("class", "robot_field");

    let robot_html = $(robot_el).children(":visible").get(0).outerHTML;
    
    console.log("html prethodnog polja", robot_html);
    console.log("djeca", $(robot_el).children());

    $(robot_el).children(":visible").remove();
    $(robot_el).children().show();

    next.children().hide()
    next.append(robot_html);

    $(robot_el).css("background-color", '');
    next.css("background-color", robot);

    //povratna vrijednost je boolean koji kaze je li se robot pomakao
    //mozemo ovo koristiti da ne brojimo poteze
    return !(col === curr_col && row == curr_row)
}