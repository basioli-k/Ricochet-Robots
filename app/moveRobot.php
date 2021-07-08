<?php
define( '__SITE_PATH', realpath( dirname( __FILE__ ) . "/.." ) );
define( '__SITE_URL', dirname( $_SERVER['PHP_SELF'] ) . "/.." );

require_once './init.php';

session_start();

if(isset($_POST["robot"]) && isset($_POST["direction"]) && isset($_SESSION["game"])){
    
    $board = $_SESSION["game"]->board;
    $robot = $_POST["robot"];
    $direction = $_POST["direction"];
    $_SESSION["game"]->players[$_POST["username"]]->move_robot( $board, $robot, $direction);
    $_SESSION["game"]->board = $board;
    header( 'Content-type:application/json;charset=utf-8' );
    
    echo json_encode(["result" => "Success"]);
    
    flush();
    exit( 0 );
}


?>