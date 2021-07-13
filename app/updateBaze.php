<?php

define( '__SITE_PATH', realpath( dirname( __FILE__ ) ) . "/.." );
define( '__SITE_URL', dirname( $_SERVER['PHP_SELF'] ) . "/.." );

require_once './init.php';

$filename1 = 'rezultati.log';
$filename2 = '../controller/usernames.log';


$polje = [];
$rezultati_string = file_get_contents($filename1);
$rezultati = explode(',', $rezultati_string);

$username_string = file_get_contents($filename2);
$usernames = explode( ',', $username_string);
$najbolji_rez = 0;

for ($i = 0; $i < count($rezultati) - 1; $i++) {
    $ime = explode(':', $rezultati[$i])[0];
    $rezultat = explode(':', $rezultati[$i])[1];
    $polje[$i] = [ "ime" => $ime, "rezultat" => (int) $rezultat, "najbolji" => false ];
    if ((int) $rezultat > $najbolji_rez)
        $najbolji_rez = (int) $rezultat;
}

for ($i = 0 ; $i < count($usernames) - 1; $i++){
    $found = false;
    for($j = 0; $j < count($polje) ; $j++){
        if ( $polje[$j]["ime"] == $usernames[$i] ){
            $found = true;
        }
    }
    if (!$found)
        array_push($polje, ["ime" => $usernames[$i], "rezultat" => 0, "najbolji" => false]);
}

for ($i = 0 ; $i < count($polje) ; $i++){
    if ($polje[$i]["rezultat"] === $najbolji_rez)
        $polje[$i]["najbolji"] = true;
}

print_r($polje);

for($i = 0; $i < count($polje) ; $i++){
    $igrac = Player::getPlayer($polje[$i]["ime"]);
    
    $igrac->games_played = (int) $igrac->games_played + 1;
    $igrac->tokens_won = (int) $igrac->tokens_won + $polje[$i]["rezultat"];
    if ($polje[$i]["najbolji"]){
        $igrac->games_won =  (int) $igrac->games_won + 1;
    }
    $igrac->update();
}


?>