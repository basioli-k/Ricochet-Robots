var ploca;

function vratiPoteze() {
    $(".board").html(ploca);
    console.log("vraćam poteze EVO ME");
}  

function zapamtiPozicije() {
    ploca = $(".board").html();
    console.log("PAMTIM POZICIJE");
}