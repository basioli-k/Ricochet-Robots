<?php

class Game{
	public $board;
	public $players;
	public $targets = array(array(RED, STAR), array(RED, PLANET), array(RED, GEAR), array(RED, MOON),
							array(BLUE, STAR), array(BLUE, PLANET), array(BLUE, GEAR), array(BLUE, MOON),
							array(GREEN, STAR), array(GREEN, PLANET), array(GREEN, GEAR), array(GREEN, MOON),
							array(YELLOW, STAR), array(YELLOW, PLANET), array(YELLOW, GEAR), array(YELLOW, MOON),
							array(BLACK, VORTEX));

	function __construct($board, $players){
		$this->board = $board;
		$this->players = $players;
	}

	function add_player($player){
		$this->players[$player->username] = $player;
	}

	function draw_board(){
		$board = $_SESSION["game"]->board;
		?>
		<table class = "board">
		<?php
		for($i = 0 ; $i < count($board); $i++){
			?>
			<tr>
			<?php
			for($j = 0 ; $j < count($board[$i]); $j++){
				$board[$i][$j]->draw_field();
			}
			?>
			</tr>
			<?php
		}
		?>
		</table>
		<?php
	}
}

?>