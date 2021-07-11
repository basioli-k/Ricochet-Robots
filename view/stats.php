
<?php require_once  __SITE_PATH . "/view/_header.php";?>

<div class="wrapper">
<div class="container">
<h1>Stats</h1>
<form action="<?php echo __SITE_URL; ?>/index.php?rt=stats" method="post">

<div class="btn_group">
    <button name="btn_stats" class="button" value="games_won">Games won</button>
    <button name="btn_stats" class="button" value="games_played">Games played</button>
    <button name="btn_stats" class="button" value="tokens_won">Tokens won</button>
</div>
</form>
<br>
<table class= "stat_table">
    <tr class="stat_row">
        <th class="stat_row_head"> Username </th>
        <th class="stat_row_head"> <?php echo $column_name; ?> </th>
    </tr>
    <?php
        foreach($players as $player){
            echo "<tr class=\"stat_row\"> <td class=\"stat_row_data\">" . $player->username . "</td>" . "<td class=\"stat_row_data\">" . $player->$sort_by . "</td>" . "</tr>";
        }
    ?>
</table>
<br>
<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <button  class="button" value="Back">Back</button>
</form>
</div>
</div>


<?php require_once __SITE_PATH . "/view/_footer.php";?>