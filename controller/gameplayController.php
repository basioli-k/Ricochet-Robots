<?php

class GameplayController{

    function index() {

        if( !isset($_SESSION["game"] ) ){
            $board = [  [new Field("1101"), new Field("0100"), new Field("0110")],
                        [new Field("1100"), new Field("0010", NULL, RED), new Field("1010")],
                        [new Field("1001"), new Field("0001"), new Field("0011")]];

            $game = new Game($board, array($_SESSION["player"]));
            
            $_SESSION["game"] = $game;
        }

        if(isset($_POST["robot"]) && isset($_POST["direction"]) && isset($_SESSION["game"])){
            $board = $_SESSION["game"]->board;
            $robot = $_POST["robot"];
            $direction = $_POST["direction"];
            $_SESSION["game"]->players[0]->move_robot( $board, $robot, $direction);
            $_SESSION["game"]->board = $board;
        }

        require_once __SITE_PATH . "/view/gameplay.php";
        
    }
}

?>