<?php 

class StatsController {

    function index(){

        if(isset($_POST["sort_by"]))
            $sort_by = $_POST["sort_by"];
        else  
            $sort_by = "games_won";

        $name_pieces = explode("_", $sort_by);
        $name_pieces[0] = ucfirst($name_pieces[0]);
        $column_name = implode(" ", $name_pieces);
        
        $players = Player::order_by($sort_by);

        require_once __SITE_PATH . "/view/stats.php";
        exit();
    }
}

?> 