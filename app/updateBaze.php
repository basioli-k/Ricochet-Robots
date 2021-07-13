<?php

$filename1 = 'rezultati.log';
$filename2 = '../controller/usernames.log';


$polje = [];
$rezultati_string = file_get_contents($filename1);
$rezultati = explode(',', $rezultati_string);

$username_string = file_get_contents($filename2);
$usernames = explode( ',',$username)

for ($i = 0; $i < count($rezultati) - 1; $i++) {
    $ime = explode(':', $rezultati[$i])[0];
    $broj = explode(':', $rezultati[$i])[1];
    $polje[$i] = array($ime, $broj);
}

print_r($usernames);

print_r($polje);



?>